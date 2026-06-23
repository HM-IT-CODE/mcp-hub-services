use anyhow::Result;
use rusqlite::{Connection, params};
use std::sync::Mutex;

use crate::models::SentinelNode;

pub struct NodeRegistry {
    db: Mutex<Connection>,
}

impl NodeRegistry {
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS nodes (
                id          TEXT PRIMARY KEY,
                name        TEXT NOT NULL,
                url         TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                active      INTEGER NOT NULL DEFAULT 0
            );"
        )?;
        Ok(Self { db: Mutex::new(conn) })
    }

    pub fn list(&self) -> Result<Vec<SentinelNode>> {
        let db = self.db.lock().unwrap();
        let mut stmt = db.prepare(
            "SELECT id, name, url, description, active FROM nodes ORDER BY name"
        )?;
        let nodes = stmt.query_map([], |row| {
            Ok(SentinelNode {
                id:          row.get(0)?,
                name:        row.get(1)?,
                url:         row.get(2)?,
                description: row.get(3)?,
                active:      row.get::<_, i32>(4)? != 0,
            })
        })?.filter_map(|r| r.ok()).collect();
        Ok(nodes)
    }

    pub fn add(&self, node: &SentinelNode) -> Result<()> {
        let db = self.db.lock().unwrap();
        db.execute(
            "INSERT INTO nodes (id, name, url, description, active) VALUES (?1,?2,?3,?4,?5)",
            params![node.id, node.name, node.url, node.description, node.active as i32],
        )?;
        Ok(())
    }

    pub fn remove(&self, id: &str) -> Result<()> {
        let db = self.db.lock().unwrap();
        db.execute("DELETE FROM nodes WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn set_active(&self, id: &str, active: bool) -> Result<()> {
        let db = self.db.lock().unwrap();
        db.execute(
            "UPDATE nodes SET active = ?1 WHERE id = ?2",
            params![active as i32, id],
        )?;
        Ok(())
    }

    pub fn get_by_id(&self, id: &str) -> Result<Option<SentinelNode>> {
        let db = self.db.lock().unwrap();
        let mut stmt = db.prepare(
            "SELECT id, name, url, description, active FROM nodes WHERE id = ?1"
        )?;
        let node = stmt.query_map(params![id], |row| {
            Ok(SentinelNode {
                id:          row.get(0)?,
                name:        row.get(1)?,
                url:         row.get(2)?,
                description: row.get(3)?,
                active:      row.get::<_, i32>(4)? != 0,
            })
        })?.next().transpose()?;
        Ok(node)
    }
}
