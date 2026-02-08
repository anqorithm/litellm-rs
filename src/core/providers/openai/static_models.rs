use super::OpenAIModelFamily;

pub(super) type StaticModelDefinition =
    (&'static str, &'static str, OpenAIModelFamily, u32, Option<u32>, f64, f64);

pub(super) fn definitions() -> Vec<StaticModelDefinition> {
    vec![
            // ==================== GPT-4O Models (2024-2025) ====================
            (
                "gpt-4o",
                "GPT-4O",
                OpenAIModelFamily::GPT4O,
                128000,
                Some(16384),
                0.0025, // $2.50/1M input
                0.010,  // $10/1M output
            ),
            (
                "gpt-4o-2024-11-20",
                "GPT-4O (Nov 2024)",
                OpenAIModelFamily::GPT4O,
                128000,
                Some(16384),
                0.0025,
                0.010,
            ),
            (
                "gpt-4o-2024-08-06",
                "GPT-4O (Aug 2024)",
                OpenAIModelFamily::GPT4O,
                128000,
                Some(16384),
                0.0025,
                0.010,
            ),
            (
                "gpt-4.1",
                "GPT-4.1",
                OpenAIModelFamily::GPT41,
                128000,
                Some(32768),
                0.002, // $2.00/1M input
                0.008, // $8.00/1M output
            ),
            (
                "gpt-4.1-mini",
                "GPT-4.1 Mini",
                OpenAIModelFamily::GPT41Mini,
                128000,
                Some(32768),
                0.0004,
                0.0016,
            ),
            (
                "gpt-4.1-nano",
                "GPT-4.1 Nano",
                OpenAIModelFamily::GPT41Nano,
                128000,
                Some(16384),
                0.0001,
                0.0004,
            ),
            // GPT-4O Mini
            (
                "gpt-4o-mini",
                "GPT-4O Mini",
                OpenAIModelFamily::GPT4OMini,
                128000,
                Some(16384),
                0.00015, // $0.15/1M input
                0.0006,  // $0.60/1M output
            ),
            (
                "gpt-4o-mini-2024-07-18",
                "GPT-4O Mini (Jul 2024)",
                OpenAIModelFamily::GPT4OMini,
                128000,
                Some(16384),
                0.00015,
                0.0006,
            ),
            // GPT-4O Audio
            (
                "gpt-4o-audio-preview",
                "GPT-4O Audio Preview",
                OpenAIModelFamily::GPT4OAudio,
                128000,
                Some(16384),
                0.0025,
                0.010,
            ),
            (
                "gpt-4o-audio-preview-2024-12-17",
                "GPT-4O Audio (Dec 2024)",
                OpenAIModelFamily::GPT4OAudio,
                128000,
                Some(16384),
                0.0025,
                0.010,
            ),
            // GPT-4O Realtime
            (
                "gpt-4o-realtime-preview",
                "GPT-4O Realtime Preview",
                OpenAIModelFamily::Realtime,
                128000,
                Some(4096),
                0.005,
                0.020,
            ),
            // ==================== O-Series Reasoning Models (2024-2025) ====================
            // O1 Models
            (
                "o1",
                "O1",
                OpenAIModelFamily::O1,
                200000,
                Some(100000),
                0.015, // $15/1M input
                0.060, // $60/1M output
            ),
            (
                "o1-2024-12-17",
                "O1 (Dec 2024)",
                OpenAIModelFamily::O1,
                200000,
                Some(100000),
                0.015,
                0.060,
            ),
            (
                "o1-preview",
                "O1 Preview",
                OpenAIModelFamily::O1,
                128000,
                Some(32768),
                0.015,
                0.060,
            ),
            (
                "o1-mini",
                "O1 Mini",
                OpenAIModelFamily::O1,
                128000,
                Some(65536),
                0.003, // $3/1M input
                0.012, // $12/1M output
            ),
            (
                "o1-mini-2024-09-12",
                "O1 Mini (Sep 2024)",
                OpenAIModelFamily::O1,
                128000,
                Some(65536),
                0.003,
                0.012,
            ),
            // O1 Pro
            (
                "o1-pro",
                "O1 Pro",
                OpenAIModelFamily::O1Pro,
                200000,
                Some(100000),
                0.150, // $150/1M input (ChatGPT Pro)
                0.600, // $600/1M output
            ),
            (
                "o1-pro-2024-12-17",
                "O1 Pro (Dec 2024)",
                OpenAIModelFamily::O1Pro,
                200000,
                Some(100000),
                0.150,
                0.600,
            ),
            // O3 Mini (2025)
            (
                "o3-mini",
                "O3 Mini",
                OpenAIModelFamily::O3Mini,
                200000,
                Some(100000),
                0.0011, // $1.10/1M input
                0.0044, // $4.40/1M output
            ),
            (
                "o3-mini-2025-01-31",
                "O3 Mini (Jan 2025)",
                OpenAIModelFamily::O3Mini,
                200000,
                Some(100000),
                0.0011,
                0.0044,
            ),
            (
                "o3-pro",
                "O3 Pro",
                OpenAIModelFamily::O3Pro,
                200000,
                Some(100000),
                0.020, // Premium reasoning tier
                0.080,
            ),
            // O4 Mini (2025)
            (
                "o4-mini",
                "O4 Mini",
                OpenAIModelFamily::O4Mini,
                200000,
                Some(100000),
                0.0011,
                0.0044,
            ),
            (
                "o4-mini-2025-04-16",
                "O4 Mini (Apr 2025)",
                OpenAIModelFamily::O4Mini,
                200000,
                Some(100000),
                0.0011,
                0.0044,
            ),
            // ==================== GPT-5 Series (2025) ====================
            // GPT-5 (August 2025)
            (
                "gpt-5",
                "GPT-5",
                OpenAIModelFamily::GPT5,
                272000,
                Some(128000),
                0.00125, // $1.25/1M input
                0.010,   // $10/1M output
            ),
            (
                "gpt-5-2025-08-01",
                "GPT-5 (Aug 2025)",
                OpenAIModelFamily::GPT5,
                272000,
                Some(128000),
                0.00125,
                0.010,
            ),
            // GPT-5 Mini
            (
                "gpt-5-mini",
                "GPT-5 Mini",
                OpenAIModelFamily::GPT5Mini,
                272000,
                Some(64000),
                0.00025, // $0.25/1M input
                0.002,   // $2/1M output
            ),
            // GPT-5 Nano
            (
                "gpt-5-nano",
                "GPT-5 Nano",
                OpenAIModelFamily::GPT5Nano,
                128000,
                Some(32000),
                0.00005, // $0.05/1M input
                0.0004,  // $0.40/1M output
            ),
            // GPT-5.1 (November 2025)
            (
                "gpt-5.1",
                "GPT-5.1",
                OpenAIModelFamily::GPT51,
                272000,
                Some(128000),
                0.00125, // $1.25/1M input
                0.010,   // $10/1M output
            ),
            (
                "gpt-5.1-2025-11-01",
                "GPT-5.1 (Nov 2025)",
                OpenAIModelFamily::GPT51,
                272000,
                Some(128000),
                0.00125,
                0.010,
            ),
            // GPT-5.1 Thinking (Reasoning mode)
            (
                "gpt-5.1-thinking",
                "GPT-5.1 Thinking",
                OpenAIModelFamily::GPT51Thinking,
                400000,
                Some(196000),
                0.00250, // $2.50/1M input (thinking mode)
                0.020,   // $20/1M output (thinking mode)
            ),
            (
                "gpt-5.1-thinking-mini",
                "GPT-5.1 Thinking Mini",
                OpenAIModelFamily::GPT51Thinking,
                400000,
                Some(128000),
                0.00125,
                0.010,
            ),
            // ==================== GPT-5.2 Series (2025 - Latest) ====================
            // GPT-5.2 Pro (Flagship)
            (
                "gpt-5.2-pro",
                "GPT-5.2 Pro",
                OpenAIModelFamily::GPT52Pro,
                400000,
                Some(128000),
                0.021, // $21/1M input
                0.168, // $168/1M output
            ),
            // GPT-5.2 (Standard)
            (
                "gpt-5.2",
                "GPT-5.2",
                OpenAIModelFamily::GPT52,
                400000,
                Some(128000),
                0.00175, // $1.75/1M input
                0.014,   // $14/1M output
            ),
            // GPT-5.2 Chat
            (
                "gpt-5.2-chat",
                "GPT-5.2 Chat",
                OpenAIModelFamily::GPT52,
                128000,
                Some(16384),
                0.00175, // $1.75/1M input
                0.014,   // $14/1M output
            ),
            // GPT-5.2 Codex (Code-optimized)
            (
                "gpt-5.2-codex",
                "GPT-5.2 Codex",
                OpenAIModelFamily::GPT52Codex,
                400000,
                Some(128000),
                0.00175, // $1.75/1M input
                0.014,   // $14/1M output
            ),
            (
                "gpt-5-codex",
                "GPT-5 Codex",
                OpenAIModelFamily::GPT52Codex,
                400000,
                Some(128000),
                0.00125, // $1.25/1M input
                0.010,   // $10/1M output
            ),
            (
                "codex-mini-latest",
                "Codex Mini Latest",
                OpenAIModelFamily::GPT52Codex,
                400000,
                Some(64000),
                0.0009,
                0.0072,
            ),
            // GPT-5.1 Codex variants
            (
                "gpt-5.1-codex",
                "GPT-5.1 Codex",
                OpenAIModelFamily::GPT51,
                400000,
                Some(128000),
                0.00125, // $1.25/1M input
                0.010,   // $10/1M output
            ),
            (
                "gpt-5.1-codex-mini",
                "GPT-5.1 Codex Mini",
                OpenAIModelFamily::GPT51,
                400000,
                Some(64000),
                0.00025, // $0.25/1M input
                0.002,   // $2/1M output
            ),
            (
                "gpt-5.1-codex-max",
                "GPT-5.1 Codex Max",
                OpenAIModelFamily::GPT51,
                400000,
                Some(128000),
                0.00125, // $1.25/1M input
                0.010,   // $10/1M output
            ),
            (
                "gpt-5.1-chat",
                "GPT-5.1 Chat",
                OpenAIModelFamily::GPT51,
                128000,
                Some(16384),
                0.00125, // $1.25/1M input
                0.010,   // $10/1M output
            ),
            // ==================== GPT Audio Models (2025) ====================
            (
                "gpt-audio",
                "GPT Audio",
                OpenAIModelFamily::GPTAudio,
                128000,
                Some(16384),
                0.0025, // $2.50/1M input
                0.010,  // $10/1M output
            ),
            (
                "gpt-audio-mini",
                "GPT Audio Mini",
                OpenAIModelFamily::GPTAudio,
                128000,
                Some(16384),
                0.0006, // $0.60/1M input
                0.0024, // $2.40/1M output
            ),
            (
                "gpt-image-1",
                "GPT Image 1",
                OpenAIModelFamily::GPTImage,
                128000,
                Some(16384),
                0.005,
                0.020,
            ),
            (
                "gpt-image-1-mini",
                "GPT Image 1 Mini",
                OpenAIModelFamily::GPTImage,
                128000,
                Some(16384),
                0.0025,
                0.010,
            ),
            (
                "gpt-image-1.5",
                "GPT Image 1.5",
                OpenAIModelFamily::GPTImage,
                128000,
                Some(16384),
                0.005,
                0.020,
            ),
            (
                "chatgpt-image-latest",
                "ChatGPT Image Latest",
                OpenAIModelFamily::GPTImage,
                128000,
                Some(16384),
                0.005,
                0.020,
            ),
            // ==================== GPT-4 Legacy Models ====================
            (
                "gpt-4",
                "GPT-4",
                OpenAIModelFamily::GPT4,
                8192,
                Some(8192),
                0.03,
                0.06,
            ),
            (
                "gpt-4-turbo",
                "GPT-4 Turbo",
                OpenAIModelFamily::GPT4Turbo,
                128000,
                Some(4096),
                0.01,
                0.03,
            ),
            (
                "gpt-4-turbo-2024-04-09",
                "GPT-4 Turbo (Apr 2024)",
                OpenAIModelFamily::GPT4Turbo,
                128000,
                Some(4096),
                0.01,
                0.03,
            ),
            // ==================== GPT-3.5 Models ====================
            (
                "gpt-3.5-turbo",
                "GPT-3.5 Turbo",
                OpenAIModelFamily::GPT35,
                16385,
                Some(4096),
                0.0005,
                0.0015,
            ),
            (
                "gpt-3.5-turbo-0125",
                "GPT-3.5 Turbo (Jan 2024)",
                OpenAIModelFamily::GPT35,
                16385,
                Some(4096),
                0.0005,
                0.0015,
            ),
            // ==================== DALL-E Models ====================
            (
                "dall-e-2",
                "DALL-E 2",
                OpenAIModelFamily::DALLE2,
                1000,
                None,
                0.02,
                0.02,
            ),
            (
                "dall-e-3",
                "DALL-E 3",
                OpenAIModelFamily::DALLE3,
                4000,
                None,
                0.04,
                0.08,
            ),
            // ==================== Embedding Models ====================
            (
                "text-embedding-ada-002",
                "Embedding Ada 002",
                OpenAIModelFamily::Embedding,
                8191,
                None,
                0.0001,
                0.0001,
            ),
            (
                "text-embedding-3-small",
                "Embedding 3 Small",
                OpenAIModelFamily::Embedding,
                8191,
                None,
                0.00002,
                0.00002,
            ),
            (
                "text-embedding-3-large",
                "Embedding 3 Large",
                OpenAIModelFamily::Embedding,
                8191,
                None,
                0.00013,
                0.00013,
            ),
            // ==================== Audio Models ====================
            (
                "whisper-1",
                "Whisper",
                OpenAIModelFamily::Whisper,
                25000000,
                None,
                0.006,
                0.006,
            ),
            (
                "tts-1",
                "TTS 1",
                OpenAIModelFamily::TTS,
                4096,
                None,
                0.015,
                0.015,
            ),
            (
                "tts-1-hd",
                "TTS 1 HD",
                OpenAIModelFamily::TTS,
                4096,
                None,
                0.03,
                0.03,
            ),
    ]
}
