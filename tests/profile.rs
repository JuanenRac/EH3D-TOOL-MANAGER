//! The cutting profile of a tool in a material, read from the machining database of EH3D-COMMON. These tests are built only with `--features common-machining`, because the database
//! is in the releases of EH3D-COMMON from the one that added the module `machining`.
#![cfg(feature = "common-machining")]

use eh3d_materials::machining::{CncOp, MachiningDatabase, Tool as Cutter};
use eh3d_tool_manager::model::{Kind, Tool};
use eh3d_tool_manager::profile::{cut_profile, database, describe, material_ids, NOT_VERIFIED};

/// A material of the database and a diameter it has a point of the cutter and the operation for.
fn point(db: &MachiningDatabase, op: CncOp, cutter: Cutter) -> Option<(String, f64, bool, f64)> {
    db.materials().iter().find_map(|w| {
        w.cnc
            .iter()
            .find(|p| p.op == op && p.tool == cutter)
            .map(|p| (w.id.clone(), p.diameter_mm, p.verified, p.feed_mm_min))
    })
}

#[test]
fn a_profile_comes_from_the_database_and_says_when_it_was_not_verified() {
    let db = database().unwrap();
    assert!(!material_ids(&db).is_empty());
    let (id, diameter, verified, feed) =
        point(&db, CncOp::Contour, Cutter::EndMill).expect("the database has an end mill point");
    let mill = Tool::new("mill", Kind::EndMill, diameter);
    let p = cut_profile(&db, &mill, &id).unwrap();
    assert_eq!(
        (p.operation, p.verified, p.material_id.as_str()),
        ("contour", verified, id.as_str())
    );
    assert!(p.rpm > 0.0 && p.depth_per_pass_mm > 0.0);
    assert!(!p.interpolated);
    assert!((p.feed_mm_min - feed).abs() < 1e-9 || p.verified != verified);
    let text = describe(&mill, &p);
    assert_eq!(text.starts_with(NOT_VERIFIED), !p.verified, "{text}");
    assert!(text.contains("spindle") && text.contains("feed") && text.contains(&id));
}

#[test]
fn a_v_bit_and_an_engraver_engrave_and_a_drill_drills() {
    let db = database().unwrap();
    for (kind, op, cutter, operation) in [
        (Kind::VBit, CncOp::Engrave, Cutter::VBit, "engrave"),
        (Kind::Engraver, CncOp::Engrave, Cutter::VBit, "engrave"),
        (Kind::Drill, CncOp::Drill, Cutter::Drill, "drill"),
    ] {
        // a kind that the data of the database has no point for is not a failure of the program
        let Some((id, diameter, _, _)) = point(&db, op, cutter) else {
            continue;
        };
        let mut tool = Tool::new("t", kind, diameter);
        tool.tip_angle_deg = Some(60.0);
        assert_eq!(cut_profile(&db, &tool, &id).unwrap().operation, operation);
    }
}

#[test]
fn what_the_table_does_not_know_is_refused_and_never_worked_out() {
    let db = database().unwrap();
    for kind in [Kind::Laser, Kind::Nozzle] {
        let e = cut_profile(&db, &Tool::new("beam", kind, 0.1), "wood/balsa").unwrap_err();
        assert!(
            e.contains("not for this kind") || e.contains("starting points for a router"),
            "{e}"
        );
    }
    let mill = Tool::new("mill", Kind::EndMill, 3.0);
    assert!(cut_profile(&db, &mill, "wood/no-such-wood").is_err());
    let (id, _, _, _) = point(&db, CncOp::Contour, Cutter::EndMill).unwrap();
    let e = cut_profile(&db, &Tool::new("giant", Kind::EndMill, 49.9), &id).unwrap_err();
    assert!(e.contains(&id), "{e}");
}

#[test]
fn the_profile_command_lists_the_materials_and_prints_the_profile_with_its_warning_first() {
    let db = database().unwrap();
    let (id, diameter, verified, _) = point(&db, CncOp::Contour, Cutter::EndMill).unwrap();
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("dd_profile");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut crib = eh3d_tool_manager::model::Crib::default();
    crib.add(Tool::new("mill", Kind::EndMill, diameter))
        .unwrap();
    crib.add(Tool::new("beam", Kind::Laser, 0.05)).unwrap();
    let file = dir.join("crib.json");
    eh3d_tool_manager::store::save(&crib, &file).unwrap();
    let run = |args: &[&str]| {
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_eh3d-tools"))
            .args(args)
            .env("EH3D_TOOLS_DIR", &dir)
            .output()
            .unwrap();
        (
            out.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )
    };
    let f = file.to_str().unwrap();
    let (code, out, _) = run(&["profile", f, "1", &id]);
    assert_eq!(code, 0);
    assert_eq!(out.starts_with(NOT_VERIFIED), !verified, "{out}");
    assert!(out.contains("spindle"));
    let (code, _, err) = run(&["profile", f, "2", &id]);
    assert!(code == 1 && err.contains("not for this kind"), "{err}");
    assert_eq!(run(&["profile", f, "1", "wood/no-such-wood"]).0, 1);
    let (code, out, _) = run(&["profile", "--materials"]);
    assert!(code == 0 && out.lines().any(|l| l == id), "{out}");
}
