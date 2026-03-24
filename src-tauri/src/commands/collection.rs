use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct OwnedTrainee {
    pub id: String,
    pub nickname: Option<String>,
    pub star_rank: u32,
    pub potential_spd: u32,
    pub potential_sta: u32,
    pub potential_pow: u32,
    pub potential_gut: u32,
    pub potential_wit: u32,
    pub inherited_skills: String, // JSON
    pub parent_slot_1: Option<String>,
    pub parent_slot_2: Option<String>,
    pub notes: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OwnedCard {
    pub id: String,
    pub card_level: u32,
    pub limit_break: u32,
    pub bond_level: u32,
    pub notes: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Collection {
    pub trainees: Vec<OwnedTrainee>,
    pub cards: Vec<OwnedCard>,
}

#[tauri::command]
pub async fn get_collection() -> Result<Collection, String> {
    // This will be implemented using tauri-plugin-sql in a real app,
    // but for now, we'll return an empty collection as a placeholder.
    Ok(Collection {
        trainees: vec![],
        cards: vec![],
    })
}

#[tauri::command]
pub async fn clear_collection() -> Result<(), String> {
    println!("Clearing whole collection...");
    Ok(())
}

#[tauri::command]
pub async fn upsert_trainee(data: OwnedTrainee) -> Result<(), String> {
    println!("Upserting trainee: {:?}", data);
    Ok(())
}

#[tauri::command]
pub async fn upsert_card(data: OwnedCard) -> Result<(), String> {
    println!("Upserting card: {:?}", data);
    Ok(())
}
