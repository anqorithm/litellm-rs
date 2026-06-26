use super::{
    Budget, BudgetAmount, BudgetAmountError, BudgetReservationError, BudgetScope, BudgetTracker,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, Mutex};
use std::thread;

fn budget(max_budget: f64) -> Budget {
    Budget::new("global", "Global", BudgetScope::Global, max_budget)
}

#[test]
fn budget_amount_rejects_invalid_inputs() {
    assert!(matches!(
        BudgetAmount::from_f64(f64::NAN),
        Err(BudgetAmountError::NonFinite)
    ));
    assert!(matches!(
        BudgetAmount::from_f64(f64::INFINITY),
        Err(BudgetAmountError::NonFinite)
    ));
    assert!(matches!(
        BudgetAmount::from_f64(-0.01),
        Err(BudgetAmountError::Negative)
    ));
}

#[test]
fn tracker_reservation_settle_refunds_unused_amount() {
    let tracker = BudgetTracker::new();
    tracker.register_budget(budget(100.0));

    let reservation = tracker.reserve_spend(&BudgetScope::Global, 10.0).unwrap();
    assert_eq!(tracker.get_current_spend(&BudgetScope::Global), 10.0);

    let result = reservation.settle(3.0).unwrap().unwrap();
    assert_eq!(result.current_spend, 3.0);
    assert_eq!(tracker.get_current_spend(&BudgetScope::Global), 3.0);
}

#[test]
fn tracker_reservation_settle_records_actual_above_reserved_amount() {
    let tracker = BudgetTracker::new();
    tracker.register_budget(budget(10.0));

    let reservation = tracker.reserve_spend(&BudgetScope::Global, 5.0).unwrap();
    let result = reservation.settle(12.0).unwrap().unwrap();

    assert_eq!(result.current_spend, 12.0);
    assert_eq!(result.new_status, super::BudgetStatus::Exceeded);
    assert_eq!(tracker.get_current_spend(&BudgetScope::Global), 12.0);
}

#[test]
fn tracker_reservation_records_spend_for_disabled_budget() {
    let tracker = BudgetTracker::new();
    let mut budget = budget(100.0);
    budget.enabled = false;
    tracker.register_budget(budget);

    let reservation = tracker.reserve_spend(&BudgetScope::Global, 10.0).unwrap();
    assert_eq!(tracker.get_current_spend(&BudgetScope::Global), 0.0);

    let result = reservation.settle(4.0).unwrap().unwrap();
    assert_eq!(result.current_spend, 4.0);
    assert_eq!(tracker.get_current_spend(&BudgetScope::Global), 4.0);
}

#[test]
fn tracker_reservation_settle_preserves_alert_transitions() {
    let tracker = BudgetTracker::new();
    tracker.register_budget(budget(10.0));

    let reservation = tracker.reserve_spend(&BudgetScope::Global, 9.0).unwrap();
    let result = reservation.settle(9.0).unwrap().unwrap();

    assert!(result.should_alert_soft_limit);
    assert!(!result.should_alert_exceeded);

    let reservation = tracker.reserve_spend(&BudgetScope::Global, 0.5).unwrap();
    let result = reservation.settle(2.0).unwrap().unwrap();

    assert!(!result.should_alert_soft_limit);
    assert!(result.should_alert_exceeded);
}

#[test]
fn tracker_reservation_cancel_and_drop_release_amount() {
    let tracker = BudgetTracker::new();
    tracker.register_budget(budget(100.0));

    tracker
        .reserve_spend(&BudgetScope::Global, 25.0)
        .unwrap()
        .cancel();
    assert_eq!(tracker.get_current_spend(&BudgetScope::Global), 0.0);

    {
        let _reservation = tracker.reserve_spend(&BudgetScope::Global, 40.0).unwrap();
        assert_eq!(tracker.get_current_spend(&BudgetScope::Global), 40.0);
    }
    assert_eq!(tracker.get_current_spend(&BudgetScope::Global), 0.0);
}

#[test]
fn tracker_reservation_rejects_invalid_and_oversized_amounts() {
    let tracker = BudgetTracker::new();
    tracker.register_budget(budget(10.0));

    assert!(matches!(
        tracker.reserve_spend(&BudgetScope::Global, f64::NAN),
        Err(BudgetReservationError::InvalidAmount(
            BudgetAmountError::NonFinite
        ))
    ));
    assert!(matches!(
        tracker.reserve_spend(&BudgetScope::Global, 11.0),
        Err(BudgetReservationError::BudgetExceeded)
    ));
    assert!(
        tracker
            .record_spend(&BudgetScope::Global, f64::INFINITY)
            .is_none()
    );
    assert_eq!(tracker.get_current_spend(&BudgetScope::Global), 0.0);
}

#[test]
fn tracker_reservation_settle_after_reset_records_actual_spend() {
    let tracker = BudgetTracker::new();
    tracker.register_budget(budget(100.0));

    let reservation = tracker.reserve_spend(&BudgetScope::Global, 10.0).unwrap();
    assert!(tracker.reset_budget(&BudgetScope::Global));
    tracker.record_spend(&BudgetScope::Global, 5.0);

    let result = reservation.settle(3.0).unwrap().unwrap();
    assert_eq!(result.current_spend, 8.0);
    assert_eq!(tracker.get_current_spend(&BudgetScope::Global), 8.0);
}

#[test]
fn tracker_reservation_cancel_after_reset_keeps_new_period_spend() {
    let tracker = BudgetTracker::new();
    tracker.register_budget(budget(100.0));

    let reservation = tracker.reserve_spend(&BudgetScope::Global, 10.0).unwrap();
    assert!(tracker.reset_budget(&BudgetScope::Global));
    tracker.record_spend(&BudgetScope::Global, 5.0);

    reservation.cancel();
    assert_eq!(tracker.get_current_spend(&BudgetScope::Global), 5.0);
}

#[test]
fn concurrent_reservations_allow_only_one_last_budget_winner() {
    let tracker = Arc::new(BudgetTracker::new());
    tracker.register_budget(budget(10.0));
    let barrier = Arc::new(Barrier::new(16));
    let winners = Arc::new(AtomicUsize::new(0));
    let reservations = Arc::new(Mutex::new(Vec::new()));

    let handles: Vec<_> = (0..16)
        .map(|_| {
            let tracker = Arc::clone(&tracker);
            let barrier = Arc::clone(&barrier);
            let winners = Arc::clone(&winners);
            let reservations = Arc::clone(&reservations);
            thread::spawn(move || {
                barrier.wait();
                if let Ok(reservation) = tracker.reserve_spend(&BudgetScope::Global, 10.0) {
                    winners.fetch_add(1, Ordering::SeqCst);
                    reservations.lock().unwrap().push(reservation);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(winners.load(Ordering::SeqCst), 1);
    assert_eq!(tracker.get_current_spend(&BudgetScope::Global), 10.0);
    reservations.lock().unwrap().clear();
    assert_eq!(tracker.get_current_spend(&BudgetScope::Global), 0.0);
}
