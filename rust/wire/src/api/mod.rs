use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct V1EncodedStateVector(Vec<u8>);

#[derive(Serialize, Deserialize)]
pub struct GetEverythingResponse {
    state_vector: V1EncodedStateVector,
}

pub const GET_EVERYTHING_ENDPOINT: &str = "/api/get_everything";
