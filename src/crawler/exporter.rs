use crate::data::loader::Database;
use std::path::Path;

pub struct Exporter;

impl Exporter {
    pub fn export_heroes_summary_csv_bytes(db: &Database) -> std::io::Result<Vec<u8>> {
        let mut buf = Vec::new();
        // Write UTF-8 BOM so Microsoft Excel on Windows parses Chinese characters correctly
        buf.extend_from_slice(b"\xEF\xBB\xBF");

        {
            let mut wtr = csv::WriterBuilder::new().from_writer(&mut buf);
            wtr.write_record([
                "英雄ID", "英雄名称", "职业定位", "评级(Tier)", "胜率", "登场率", "禁用率", "总场次", "KDA", "场均补刀",
            ])?;

            for h in &db.heroes_summary {
                wtr.write_record(&[
                    h.id.clone(),
                    h.name.clone(),
                    h.roles.join("/"),
                    h.tier.map(|t| t.to_string()).unwrap_or_else(|| "5".to_string()),
                    format!("{:.2}%", h.win_rate * 100.0),
                    format!("{:.2}%", h.pick_rate * 100.0),
                    format!("{:.2}%", h.ban_rate * 100.0),
                    h.games.to_string(),
                    format!("{:.2}", h.kda),
                    format!("{:.1}", h.avg_cs),
                ])?;
            }

            wtr.flush()?;
        }

        Ok(buf)
    }

    pub fn export_augments_summary_csv_bytes(db: &Database) -> std::io::Result<Vec<u8>> {
        let mut buf = Vec::new();
        // Write UTF-8 BOM so Microsoft Excel on Windows parses Chinese characters correctly
        buf.extend_from_slice(b"\xEF\xBB\xBF");

        {
            let mut wtr = csv::WriterBuilder::new().from_writer(&mut buf);
            wtr.write_record([
                "强化ID", "名称", "品阶", "层级(Tier)", "胜率", "选择率", "总场次", "胜场", "HexScore评分", "描述",
            ])?;

            for a in &db.augments_summary {
                wtr.write_record(&[
                    a.id.clone(),
                    a.name.clone(),
                    a.rarity.clone(),
                    a.tier.map(|t| t.to_string()).unwrap_or_default(),
                    format!("{:.2}%", a.win_rate * 100.0),
                    format!("{:.2}%", a.pick_rate * 100.0),
                    a.games.to_string(),
                    a.wins.to_string(),
                    format!("{:.1}", a.hex_score),
                    a.description.clone(),
                ])?;
            }

            wtr.flush()?;
        }

        Ok(buf)
    }

    pub fn export_heroes_summary_csv(db: &Database, out_path: &Path) -> std::io::Result<()> {
        if let Some(parent) = out_path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let bytes = Self::export_heroes_summary_csv_bytes(db)?;
        std::fs::write(out_path, bytes)?;
        Ok(())
    }

    pub fn export_augments_summary_csv(db: &Database, out_path: &Path) -> std::io::Result<()> {
        if let Some(parent) = out_path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let bytes = Self::export_augments_summary_csv_bytes(db)?;
        std::fs::write(out_path, bytes)?;
        Ok(())
    }
}
