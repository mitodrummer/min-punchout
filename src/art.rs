//! Fighter rigs. A fighter is drawn procedurally from a palette and a pose,
//! so a new pose is a handful of coordinates, not a new drawing.

use crate::pixel::{Canvas, Rgb};
use crate::sim::{Action, Fighter, Side, State};

#[derive(Clone, Copy)]
pub struct Palette {
    pub skin: Rgb,
    pub skin_dark: Rgb,
    pub hair: Rgb,
    pub outline: Rgb,
    pub top: Rgb,
    pub top_dark: Rgb,
    pub trunks: Rgb,
    pub band: Rgb,
    pub glove: Rgb,
    pub glove_dark: Rgb,
    pub cuff: Rgb,
    pub chain: Rgb,
    pub chain_dark: Rgb,
    pub logo: Rgb,
    pub gold: Rgb,
}

pub const CHAINGUARD: Palette = Palette {
    skin: Rgb::hex(0xe8b88a),
    skin_dark: Rgb::hex(0xc9955f),
    hair: Rgb::hex(0x2b2b35),
    outline: Rgb::hex(0x14141c),
    top: Rgb::hex(0x6e4ff2),
    top_dark: Rgb::hex(0x4b33b5),
    trunks: Rgb::hex(0x2a2550),
    band: Rgb::hex(0xf4f4f8),
    glove: Rgb::hex(0xd62839),
    glove_dark: Rgb::hex(0x8d1b26),
    cuff: Rgb::hex(0xf4f4f8),
    chain: Rgb::hex(0xc9ced8),
    chain_dark: Rgb::hex(0x7d8693),
    logo: Rgb::hex(0xffffff),
    gold: Rgb::hex(0xf2c14e),
};

pub const MINIMAL: Palette = Palette {
    skin: Rgb::hex(0xf0c9a0),
    skin_dark: Rgb::hex(0xd1a476),
    hair: Rgb::hex(0x3a2a1f),
    outline: Rgb::hex(0x14141c),
    top: Rgb::hex(0x1b1b1f),
    top_dark: Rgb::hex(0x0d0d10),
    trunks: Rgb::hex(0x1b1b1f),
    band: Rgb::hex(0xffffff),
    glove: Rgb::hex(0x2ec4b6),
    glove_dark: Rgb::hex(0x1b8a80),
    cuff: Rgb::hex(0xffffff),
    chain: Rgb::hex(0xffffff),
    chain_dark: Rgb::hex(0xaaaaaa),
    logo: Rgb::hex(0xffffff),
    gold: Rgb::hex(0xf2c14e),
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Face {
    Normal,
    Focus,
    Hurt,
    Dizzy,
    Smug,
}

#[derive(Clone, Copy)]
pub enum Prop {
    /// Briefcase held at (x, y), top-left in rig coordinates.
    Briefcase(i32, i32),
}

/// A glove at (x, y) rig coordinates with a size multiplier. Larger reads as
/// closer to the viewer.
#[derive(Clone, Copy)]
pub struct Glove(pub i32, pub i32, pub f32);

#[derive(Clone, Copy)]
pub struct Pose {
    pub body_dx: i32,
    pub body_dy: i32,
    pub head_dx: i32,
    pub head_dy: i32,
    pub face: Face,
    pub left: Glove,
    pub right: Glove,
    pub prop: Option<Prop>,
}

const GUARD: Pose = Pose {
    body_dx: 0,
    body_dy: 0,
    head_dx: 0,
    head_dy: 0,
    face: Face::Normal,
    left: Glove(6, 26, 1.0),
    right: Glove(38, 26, 1.0),
    prop: None,
};

/// Opponent pose for the current sim state. Rig box is 44 wide, 56 tall.
pub fn opp_pose(fi: &Fighter, tick: u64) -> Pose {
    use Action::*;
    let bob = ((tick / 8) % 2) as i32; // idle breathing
    if fi.hit_flash > 0 && !matches!(fi.state, State::Stunned(_)) {
        return Pose { body_dx: 2, head_dx: 4, head_dy: 1, face: Face::Hurt, left: Glove(6, 30, 1.0), right: Glove(40, 28, 1.0), ..GUARD };
    }
    match fi.state {
        State::Idle => Pose { body_dy: bob, head_dy: bob, left: Glove(6, 26 + bob, 1.0), right: Glove(38, 26 + bob, 1.0), ..GUARD },
        State::Startup(Jab, _) => Pose { face: Face::Focus, right: Glove(41, 8, 1.0), ..GUARD },
        State::Recovery(Jab, _) => Pose { face: Face::Focus, body_dx: -1, right: Glove(27, 30, 1.9), ..GUARD },
        State::Startup(Hook, _) => Pose { body_dx: 4, head_dx: 5, face: Face::Focus, left: Glove(-6, 16, 1.1), right: Glove(38, 24, 1.0), ..GUARD },
        State::Recovery(Hook, _) => Pose { body_dx: -2, head_dx: -3, face: Face::Focus, left: Glove(30, 12, 1.7), right: Glove(36, 26, 1.0), ..GUARD },
        State::Startup(Uppercut, _) => Pose { body_dy: 5, head_dy: 7, face: Face::Focus, left: Glove(8, 44, 1.0), right: Glove(36, 46, 1.1), ..GUARD },
        State::Recovery(Uppercut, _) => Pose { body_dy: -2, head_dy: -3, face: Face::Focus, left: Glove(8, 30, 1.0), right: Glove(24, -4, 1.6), ..GUARD },
        State::Startup(Special, _) => Pose { face: Face::Smug, left: Glove(10, 24, 1.0), right: Glove(38, 6, 1.0), prop: Some(Prop::Briefcase(31, 0)), ..GUARD },
        State::Recovery(Special, _) => Pose { body_dy: -1, face: Face::Smug, left: Glove(4, 30, 1.0), right: Glove(22, 24, 2.8), ..GUARD },
        State::Blocking(_) => Pose { head_dy: 1, face: Face::Focus, left: Glove(16, 7, 1.1), right: Glove(28, 7, 1.1), ..GUARD },
        State::Dodging(Side::Left, _) => Pose { body_dx: -4, head_dx: -6, ..GUARD },
        State::Dodging(Side::Right, _) => Pose { body_dx: 4, head_dx: 6, ..GUARD },
        State::Stunned(_) => {
            let sway = (((tick as f32) / 4.0).sin() * 3.0) as i32;
            Pose { body_dx: sway / 2, head_dx: sway, head_dy: 2, face: Face::Dizzy, left: Glove(4, 40, 1.0), right: Glove(40, 42, 1.0), ..GUARD }
        }
    }
}

/// Draw the big opponent rig with its top-left at (ox, oy).
pub fn draw_opponent(c: &mut Canvas, ox: i32, oy: i32, p: &Palette, pose: &Pose, tick: u64) {
    let bx = ox + pose.body_dx;
    let by = oy + pose.body_dy;
    let o = p.outline;

    // Legs and boots.
    for lx in [13, 24] {
        c.rect_o(bx + lx, by + 41, 7, 11, p.skin, o);
        c.rect_o(bx + lx - 1, by + 51, 9, 5, p.trunks, o);
        c.rect(bx + lx - 1, by + 51, 9, 1, p.top); // boot trim in brand purple
    }
    // Trunks with waistband and the chest-thumping claim.
    c.rect_o(bx + 11, by + 33, 22, 9, p.trunks, o);
    c.rect(bx + 11, by + 33, 22, 2, p.band);
    // Torso, shoulders, tank top.
    c.rect_o(bx + 10, by + 17, 24, 17, p.skin, o);
    c.ellipse_o(bx + 11, by + 20, 3.5, 3.0, p.skin, o);
    c.ellipse_o(bx + 33, by + 20, 3.5, 3.0, p.skin, o);
    c.rect(bx + 13, by + 19, 18, 15, p.top);
    c.rect(bx + 13, by + 19, 1, 15, p.top_dark);
    c.rect(bx + 30, by + 19, 1, 15, p.top_dark);
    c.rect(bx + 14, by + 16, 3, 4, p.top);
    c.rect(bx + 27, by + 16, 3, 4, p.top);
    // The claim on the chest, where the player's head cannot hide it, and the
    // chain-link logo under it.
    let w = Canvas::text_width("0 CVE", 1);
    c.text(bx + 22 - w / 2, by + 21, "0 CVE", 1, p.logo);
    chain_link(c, bx + 22, by + 30, p.logo, p.top);
    // The chain itself, around the neck.
    for x in 12..=32 {
        let d = (x - 22) as i32;
        let y = by + 17 + (d * d) / 30;
        let col = if (x / 2) % 2 == 0 { p.chain } else { p.chain_dark };
        c.rect(bx + x, y, 1, 2, col);
    }
    // Neck.
    c.rect(bx + 20, by + 13, 5, 5, p.skin_dark);

    // Head.
    let hx = ox + pose.head_dx + 22;
    let hy = oy + pose.head_dy + 8;
    c.ellipse_o(hx, hy, 6.5, 6.5, p.skin, o);
    c.ellipse(hx - 7, hy, 1.5, 2.0, p.skin_dark);
    c.ellipse(hx + 7, hy, 1.5, 2.0, p.skin_dark);
    c.ellipse(hx, hy - 4, 6.0, 2.5, p.hair);
    face(c, hx, hy, pose.face, o, p.band, tick);

    // Prop, behind the glove that holds it.
    if let Some(Prop::Briefcase(px, py)) = pose.prop {
        briefcase(c, ox + px, oy + py, p);
    }

    // Arms and gloves, drawn last so they sit in front of the body.
    let (lsx, lsy) = (bx + 11, by + 21);
    let (rsx, rsy) = (bx + 33, by + 21);
    arm(c, lsx, lsy, ox + pose.left.0, oy + pose.left.1, pose.left.2, p);
    arm(c, rsx, rsy, ox + pose.right.0, oy + pose.right.1, pose.right.2, p);
}

fn chain_link(c: &mut Canvas, cx: i32, cy: i32, ink: Rgb, bg: Rgb) {
    for dx in [-3, 3] {
        c.ellipse(cx + dx, cy, 3.6, 2.4, ink);
        c.ellipse(cx + dx, cy, 2.0, 1.0, bg);
    }
}

fn briefcase(c: &mut Canvas, x: i32, y: i32, p: &Palette) {
    c.rect(x + 4, y, 5, 2, p.outline);
    c.rect_o(x, y + 2, 13, 9, p.gold, p.outline);
    c.rect(x, y + 6, 13, 1, p.outline);
    c.text(x + 5, y + 4, "$", 1, p.outline);
}

fn face(c: &mut Canvas, hx: i32, hy: i32, f: Face, ink: Rgb, white: Rgb, tick: u64) {
    let (lx, rx, ey) = (hx - 3, hx + 3, hy - 1);
    match f {
        Face::Normal => {
            c.rect(lx - 1, ey, 2, 2, ink);
            c.rect(rx, ey, 2, 2, ink);
            c.rect(hx - 1, hy + 4, 3, 1, ink);
        }
        Face::Focus => {
            c.rect(lx - 1, ey - 1, 2, 1, ink);
            c.rect(rx, ey - 1, 2, 1, ink);
            c.rect(lx - 1, ey, 2, 2, ink);
            c.rect(rx, ey, 2, 2, ink);
            c.rect(hx - 2, hy + 4, 5, 1, ink);
        }
        Face::Smug => {
            c.rect(lx - 1, ey, 2, 1, ink);
            c.rect(rx, ey, 2, 1, ink);
            c.rect(hx - 1, hy + 4, 3, 1, ink);
            c.rect(hx + 2, hy + 3, 1, 1, ink);
        }
        Face::Hurt => {
            c.rect(lx - 2, ey - 1, 1, 1, ink);
            c.rect(lx - 1, ey, 1, 1, ink);
            c.rect(lx, ey + 1, 1, 1, ink);
            c.rect(rx + 1, ey - 1, 1, 1, ink);
            c.rect(rx, ey, 1, 1, ink);
            c.rect(rx - 1, ey + 1, 1, 1, ink);
            c.ellipse_o(hx, hy + 4, 1.5, 1.2, ink, ink);
        }
        Face::Dizzy => {
            for ex in [lx, rx] {
                c.ellipse(ex, ey, 1.8, 1.8, ink);
                c.rect(ex, ey, 1, 1, white);
            }
            c.rect(hx - 1, hy + 4, 3, 1, ink);
            // Orbiting stars.
            for k in 0..3 {
                let a = (tick as f32) / 6.0 + k as f32 * 2.1;
                let sx = hx + (a.cos() * 10.0) as i32;
                let sy = hy - 4 + (a.sin() * 3.0) as i32;
                c.text(sx, sy, "*", 1, Rgb::hex(0xf2c14e));
            }
        }
    }
}

fn arm(c: &mut Canvas, sx: i32, sy: i32, gx: i32, gy: i32, scale: f32, p: &Palette) {
    c.line_o(sx, sy, gx, gy, 4.0, p.skin, p.outline);
    // Cuff where the arm meets the glove.
    let dx = (sx - gx) as f32;
    let dy = (sy - gy) as f32;
    let len = (dx * dx + dy * dy).sqrt().max(1.0);
    let cx = gx + (dx / len * 4.0 * scale) as i32;
    let cy = gy + (dy / len * 3.0 * scale) as i32;
    c.ellipse_o(cx, cy, 2.5 * scale, 2.0 * scale, p.cuff, p.outline);
    c.ellipse_o(gx, gy, 4.5 * scale, 3.6 * scale, p.glove, p.outline);
    c.ellipse(gx + (1.5 * scale) as i32, gy + (1.2 * scale) as i32, 2.0 * scale, 1.2 * scale, p.glove_dark);
    c.ellipse(gx - (1.5 * scale) as i32, gy - (1.3 * scale) as i32, 1.2 * scale, 0.7 * scale, p.cuff.mix(p.glove, 0.5));
}

/// Player pose for the current sim state. Rig box is 24 wide, 26 tall.
pub fn player_pose(fi: &Fighter, tick: u64) -> Pose {
    use Action::*;
    let bob = ((tick / 8) % 2) as i32;
    let base = Pose {
        body_dx: 0,
        body_dy: 0,
        head_dx: 0,
        head_dy: 0,
        face: Face::Normal,
        left: Glove(3, 9, 1.0),
        right: Glove(21, 9, 1.0),
        prop: None,
    };
    match fi.state {
        State::Idle => Pose { body_dy: bob, head_dy: bob, left: Glove(3, 9 + bob, 1.0), right: Glove(21, 9 + bob, 1.0), ..base },
        State::Startup(Jab, _) => Pose { right: Glove(22, 14, 0.9), ..base },
        State::Recovery(Jab, _) => Pose { body_dy: -1, right: Glove(17, -4, 0.8), ..base },
        State::Startup(Hook, _) => Pose { body_dx: -2, left: Glove(-3, 12, 1.0), ..base },
        State::Recovery(Hook, _) => Pose { body_dx: 2, left: Glove(14, -2, 0.85), ..base },
        State::Startup(Uppercut, _) | State::Startup(Special, _) => Pose { body_dy: 3, head_dy: 3, right: Glove(20, 20, 1.0), left: Glove(4, 18, 1.0), ..base },
        State::Recovery(Uppercut, _) | State::Recovery(Special, _) => Pose { body_dy: -2, head_dy: -2, right: Glove(12, -7, 0.9), ..base },
        State::Blocking(_) => Pose { left: Glove(8, 1, 1.0), right: Glove(16, 1, 1.0), ..base },
        State::Dodging(Side::Left, _) => Pose { body_dx: -9, head_dx: -12, left: Glove(-6, 10, 1.0), right: Glove(12, 10, 1.0), ..base },
        State::Dodging(Side::Right, _) => Pose { body_dx: 9, head_dx: 12, left: Glove(12, 10, 1.0), right: Glove(30, 10, 1.0), ..base },
        State::Stunned(_) => {
            let sway = (((tick as f32) / 4.0).sin() * 2.0) as i32;
            Pose { body_dx: sway, head_dx: sway * 2, left: Glove(2, 18, 1.0), right: Glove(22, 18, 1.0), ..base }
        }
    }
}

/// Draw Minimal from behind with the rig's top-left at (ox, oy).
pub fn draw_player(c: &mut Canvas, ox: i32, oy: i32, p: &Palette, pose: &Pose) {
    let bx = ox + pose.body_dx;
    let by = oy + pose.body_dy;
    let o = p.outline;
    // Trunks with a white band and a tiny "m" on the band.
    c.rect_o(bx + 5, by + 21, 14, 5, p.trunks, o);
    c.rect(bx + 5, by + 21, 14, 1, p.band);
    // Back and shoulders.
    c.rect_o(bx + 5, by + 11, 14, 11, p.skin, o);
    c.ellipse_o(bx + 6, by + 13, 3.0, 2.5, p.skin, o);
    c.ellipse_o(bx + 18, by + 13, 3.0, 2.5, p.skin, o);
    c.rect(bx + 11, by + 11, 2, 11, p.skin_dark); // spine
    // Head from behind: all hair.
    let hx = ox + pose.head_dx + 12;
    let hy = oy + pose.head_dy + 6;
    c.rect(hx - 2, hy + 4, 4, 3, p.skin_dark);
    c.ellipse_o(hx, hy, 5.5, 5.5, p.hair, o);
    c.ellipse(hx - 1, hy - 2, 3.0, 2.0, p.hair.mix(Rgb::hex(0xffffff), 0.12));
    // Arms and gloves.
    arm(c, bx + 6, by + 13, ox + pose.left.0, oy + pose.left.1, pose.left.2 * 0.75, p);
    arm(c, bx + 18, by + 13, ox + pose.right.0, oy + pose.right.1, pose.right.2 * 0.75, p);
}
