pub mod encounter_state;
mod entity_tracker;
mod id_tracker;
pub mod models;
mod party_tracker;
mod status_tracker;

use crate::parser::encounter_state::EncounterState;
use crate::parser::entity_tracker::{get_current_and_max_hp, EntityTracker};
use crate::parser::id_tracker::IdTracker;
use crate::parser::models::EntityType;
use crate::parser::party_tracker::PartyTracker;
use crate::parser::status_tracker::{StatusEffectTargetType, StatusTracker};
use anyhow::Result;
use chrono::Utc;
use meter_core::packets::definitions::*;
use meter_core::packets::opcodes::Pkt;
use meter_core::start_capture;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{Manager, Window, Wry};
use tokio::runtime::Runtime;

pub fn start(window: Window<Wry>, ip: String, port: u16) -> Result<()> {
    let id_tracker = Rc::new(RefCell::new(IdTracker::new()));
    let party_tracker = Rc::new(RefCell::new(PartyTracker::new(id_tracker.clone())));
    let status_tracker = Rc::new(RefCell::new(StatusTracker::new(party_tracker.clone())));
    let mut entity_tracker = EntityTracker::new(
        status_tracker.clone(),
        id_tracker.clone(),
        party_tracker.clone(),
    );
    let mut state = EncounterState::new(window.clone());
    let rt = Runtime::new().unwrap();
    let _guard = rt.enter();
    let rx = match start_capture(ip, port) {
        Ok(rx) => rx,
        Err(e) => {
            println!("Error starting capture: {}", e);
            return Ok(());
        }
    };
    let mut last_update = Instant::now();
    let duration = Duration::from_millis(100);

    let reset = Arc::new(Mutex::new(false));
    let pause = Arc::new(Mutex::new(false));
    let reset_clone = reset.clone();
    let pause_clone = pause.clone();
    let meter_window_clone = window.clone();
    window.listen_global("reset-request", move |_event| {
        if let Ok(ref mut reset) = reset_clone.try_lock() {
            **reset = true;
            meter_window_clone.emit("reset-encounter", "").ok();
        }
    });
    let meter_window_clone = window.clone();
    window.listen_global("pause-request", move |_event| {
        if let Ok(ref mut pause) = pause_clone.try_lock() {
            **pause = !(**pause);
            meter_window_clone.emit("pause-encounter", "").ok();
        }
    });

    while let Ok((op, data)) = rx.recv() {
        if let Ok(ref mut reset) = reset.try_lock() {
            if **reset {
                state.soft_reset();
                **reset = false;
            }
        }
        if let Ok(ref mut pause) = pause.try_lock() {
            if **pause {
                continue;
            }
        }
        match op {
            Pkt::CounterAttackNotify => {
                let pkt = PKTCounterAttackNotify::new(&data)?;
                if let Some(entity) = entity_tracker.entities.get(&pkt.source_id) {
                    state.on_counterattack(entity);
                }
            }
            Pkt::DeathNotify => {
                let pkt = PKTDeathNotify::new(&data)?;
                debug_print("", &pkt);
                if let Some(entity) = entity_tracker.entities.get(&pkt.target_id) {
                    state.on_death(entity);
                }
            }
            Pkt::IdentityGaugeChangeNotify => {
                let pkt = PKTIdentityGaugeChangeNotify::new(&data)?;
                state.on_identity_gain(pkt);
            }
            Pkt::InitEnv => {
                let pkt = PKTInitEnv::new(&data)?;
                let entity = entity_tracker.init_env(pkt);
                debug_print("init env", &entity);
                state.on_init_env(entity);
            }
            Pkt::InitPC => {
                let pkt = PKTInitPC::new(&data)?;
                let (hp, max_hp) = get_current_and_max_hp(&pkt.stat_pair);
                let entity = entity_tracker.init_pc(pkt);
                debug_print("init pc", &entity);

                state.on_init_pc(entity, hp, max_hp)
            }
            Pkt::MigrationExecute => {
                let pkt = PKTMigrationExecute::new(&data)?;
                entity_tracker.migration_execute(pkt);
            }
            Pkt::NewPC => {
                let pkt = PKTNewPC::new(&data)?;
                let (hp, max_hp) = get_current_and_max_hp(&pkt.pc_struct.stat_pair);
                let entity = entity_tracker.new_pc(pkt);
                debug_print("new pc", &entity);
                state.on_new_pc(entity, hp, max_hp);
            }
            Pkt::NewNpc => {
                let pkt = PKTNewNpc::new(&data)?;
                let (hp, max_hp) = get_current_and_max_hp(&pkt.npc_data.stat_pair);
                let entity = entity_tracker.new_npc(pkt, max_hp);
                debug_print("new npc", &entity);
                state.on_new_npc(entity, hp, max_hp);
            }
            Pkt::NewNpcSummon => {
                let pkt = PKTNewNpcSummon::new(&data)?;
                let (hp, max_hp) = get_current_and_max_hp(&pkt.npc_data.stat_pair);
                let entity = entity_tracker.new_npc_summon(pkt, max_hp);
                debug_print("new summon", &entity);
                state.on_new_npc(entity, hp, max_hp);
            }
            Pkt::NewProjectile => {
                let pkt = PKTNewProjectile::new(&data)?;
                entity_tracker.new_projectile(pkt);
            }
            Pkt::ParalyzationStateNotify => {
                let pkt = PKTParalyzationStateNotify::new(&data)?;
                state.on_stagger_change(pkt);
            }
            Pkt::PartyInfo => {
                let pkt = PKTPartyInfo::new(&data)?;
                entity_tracker.party_info(pkt);
                let local_player_id = entity_tracker.local_player_id;
                if let Some(entity) = entity_tracker.entities.get(&local_player_id) {
                    state.update_local_player(&entity.name);
                }
            }
            Pkt::PartyLeaveResult => {
                let pkt = PKTPartyLeaveResult::new(&data)?;
                party_tracker
                    .borrow_mut()
                    .remove(pkt.party_instance_id, pkt.name);
            }
            Pkt::PartyStatusEffectAddNotify => {
                let pkt = PKTPartyStatusEffectAddNotify::new(&data)?;
                entity_tracker.party_status_effect_add(pkt);
            }
            Pkt::PartyStatusEffectRemoveNotify => {
                let pkt = PKTPartyStatusEffectRemoveNotify::new(&data)?;
                entity_tracker.party_status_effect_remove(pkt);
            }
            Pkt::PartyStatusEffectResultNotify => {
                let pkt = PKTPartyStatusEffectResultNotify::new(&data)?;
                party_tracker.borrow_mut().add(
                    pkt.raid_instance_id,
                    pkt.party_instance_id,
                    pkt.character_id,
                    0,
                    None,
                );
            }
            Pkt::RaidBossKillNotify => {
                state.on_phase_transition(1);
            }
            Pkt::RaidResult => {
                state.on_phase_transition(0);
            }
            Pkt::RemoveObject => {
                let pkt = PKTRemoveObject::new(&data)?;
                for upo in pkt.unpublished_objects {
                    status_tracker
                        .borrow_mut()
                        .remove_local_object(upo.object_id);
                }
            }
            Pkt::SkillCastNotify => {
                // identity skills
                // idk if i want to use this
                // only gets sent on certain identity casts
                // e.g. arcana cards, sorc ignite (only turning off)
                // let pkt = PKTSkillCastNotify::new(&data)?;
                // let mut entity = entity_tracker.get_source_entity(pkt.caster);
                // entity = entity_tracker.guess_is_player(entity, pkt.skill_id);
                // println!("skill cast notify {:?}", &entity);
                // parser.on_skill_start(entity, pkt.skill_id as i32, Utc::now().timestamp_millis());
            }
            Pkt::SkillStartNotify => {
                let pkt = PKTSkillStartNotify::new(&data)?;
                let mut entity = entity_tracker.get_source_entity(pkt.source_id);
                entity = entity_tracker.guess_is_player(entity, pkt.skill_id);
                state.on_skill_start(entity, pkt.skill_id as i32, Utc::now().timestamp_millis());
            }
            Pkt::SkillStageNotify => {
                // let pkt = PKTSkillStageNotify::new(&data);
            }
            Pkt::SkillDamageAbnormalMoveNotify => {
                let pkt = PKTSkillDamageAbnormalMoveNotify::new(&data)?;
                let owner = entity_tracker.get_source_entity(pkt.source_id);
                let local_character_id = id_tracker
                    .borrow()
                    .get_local_character_id(entity_tracker.local_player_id);
                for event in pkt.skill_damage_abnormal_move_events.iter() {
                    let target_entity =
                        entity_tracker.get_or_create_entity(event.skill_damage_event.target_id);
                    let source_entity = entity_tracker.get_or_create_entity(pkt.source_id);
                    let (se_on_source, se_on_target) = status_tracker
                        .borrow_mut()
                        .get_status_effects(&owner, &target_entity, local_character_id);
                    state.on_damage(
                        &owner,
                        &source_entity,
                        &target_entity,
                        event.skill_damage_event.damage,
                        pkt.skill_id as i32,
                        pkt.skill_effect_id as i32,
                        event.skill_damage_event.modifier as i32,
                        event.skill_damage_event.cur_hp,
                        event.skill_damage_event.max_hp,
                        se_on_source,
                        se_on_target,
                    );
                }
            }
            Pkt::SkillDamageNotify => {
                let pkt = PKTSkillDamageNotify::new(&data)?;
                let owner = entity_tracker.get_source_entity(pkt.source_id);
                let local_character_id = id_tracker
                    .borrow()
                    .get_local_character_id(entity_tracker.local_player_id);
                for event in pkt.skill_damage_events.iter() {
                    let target_entity = entity_tracker.get_or_create_entity(event.target_id);
                    // source_entity is to determine battle item
                    let source_entity = entity_tracker.get_or_create_entity(pkt.source_id);
                    let (se_on_source, se_on_target) = status_tracker
                        .borrow_mut()
                        .get_status_effects(&owner, &target_entity, local_character_id);
                    state.on_damage(
                        &owner,
                        &source_entity,
                        &target_entity,
                        event.damage,
                        pkt.skill_id as i32,
                        pkt.skill_effect_id as i32,
                        event.modifier as i32,
                        event.cur_hp,
                        event.max_hp,
                        se_on_source,
                        se_on_target,
                    );
                }
            }
            Pkt::StatusEffectAddNotify => {
                let pkt = PKTStatusEffectAddNotify::new(&data)?;
                entity_tracker
                    .build_and_register_status_effect(&pkt.status_effect_data, pkt.object_id)
            }
            Pkt::StatusEffectDurationNotify => {
                let pkt = PKTStatusEffectDurationNotify::new(&data)?;
                status_tracker.borrow_mut().update_status_duration(
                    pkt.effect_instance_id,
                    pkt.target_id,
                    pkt.expiration_tick,
                    StatusEffectTargetType::Local,
                );
            }
            Pkt::StatusEffectRemoveNotify => {
                let pkt = PKTStatusEffectRemoveNotify::new(&data)?;
                status_tracker.borrow_mut().remove_status_effects(
                    pkt.object_id,
                    pkt.status_effect_ids,
                    StatusEffectTargetType::Local,
                );
            }
            Pkt::TriggerBossBattleStatus => {
                state.on_phase_transition(2);
            }
            Pkt::TriggerStartNotify => {
                // let pkt = PKTTriggerStartNotify::new(&data)?;
            }
            Pkt::ZoneObjectUnpublishNotify => {
                let pkt = PKTZoneObjectUnpublishNotify::new(&data)?;
                status_tracker
                    .borrow_mut()
                    .remove_local_object(pkt.object_id);
            }
            Pkt::StatusEffectSyncDataNotify => {
                // let pkt = PKTStatusEffectSyncDataNotify::new(&data);
                // shields
            }
            Pkt::TroopMemberUpdateMinNotify => {
                // let pkt = PKTTroopMemberUpdateMinNotify::new(&data);
                // shields
            }
            _ => {
                continue;
            }
        }

        if last_update.elapsed() >= duration || state.raid_end {
            let mut clone = state.encounter.clone();
            let window = window.clone();
            tokio::task::spawn(async move {
                if !clone.current_boss_name.is_empty() {
                    clone.current_boss = clone.entities.get(&clone.current_boss_name).cloned();
                    if clone.current_boss.is_none() {
                        clone.current_boss_name = String::new();
                    }
                }
                clone.entities.retain(|_, e| {
                    (e.entity_type == EntityType::PLAYER || e.entity_type == EntityType::ESTHER)
                        && e.damage_stats.damage_dealt > 0
                        && e.max_hp > 0
                });
                if !clone.entities.is_empty() {
                    window
                        .emit("encounter-update", Some(clone))
                        .expect("failed to emit encounter-update");
                }
            });

            last_update = Instant::now();
        }

        if state.raid_end {
            state.soft_reset();
            state.raid_end = false;
            state.saved = false;
        }
    }

    Ok(())
}

fn debug_print<T: Debug>(desc: &str, x: &T) {
    #[cfg(debug_assertions)]
    {
        println!("{:?}|{}: {:?}", Utc::now(), desc, x);
    }
}
