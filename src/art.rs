//! Fighter rigs. A fighter is drawn procedurally from a look (palette, build,
//! hair, logo) and a pose, so a new opponent is a palette and a few numbers,
//! and a new pose is a handful of coordinates.

use crate::pixel::{Canvas, Rgb};
use crate::sim::{Action, Fighter, OppId, Side, State};

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

const BASE: Palette = Palette {
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
    top: Rgb::hex(0x1b1b1f),
    top_dark: Rgb::hex(0x0d0d10),
    trunks: Rgb::hex(0x1b1b1f),
    band: Rgb::hex(0xffffff),
    glove: Rgb::hex(0x2ec4b6),
    glove_dark: Rgb::hex(0x1b8a80),
    cuff: Rgb::hex(0xffffff),
    ..BASE
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Hair {
    Buzz,
    Bald,
    Cap,
    Hoodie,
    Wizard,
    Curly,
    Spiky,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Logo {
    Chain,
    Whale,
    Cat,
    Shield,
    Circle,
    Cube,
    Flake,
}

/// Everything about how an opponent is drawn.
#[derive(Clone, Copy)]
pub struct Look {
    pub p: Palette,
    /// Extra torso width in pixels; negative is thinner.
    pub torso_dw: i32,
    pub head_r: f32,
    pub hair: Hair,
    pub logo: Logo,
    pub chest: &'static str,
    /// Chain around the neck.
    pub necklace: bool,
    pub banner: Rgb,
}

pub fn look(id: OppId) -> Look {
    match id {
        OppId::Codespaces => Look {
            p: Palette {
                top: Rgb::hex(0x24292f),
                top_dark: Rgb::hex(0x0d1117),
                trunks: Rgb::hex(0x1f6feb),
                glove: Rgb::hex(0x8957e5),
                glove_dark: Rgb::hex(0x5a3aa0),
                hair: Rgb::hex(0x24292f),
                ..BASE
            },
            torso_dw: 0,
            head_r: 6.5,
            hair: Hair::Hoodie,
            logo: Logo::Cat,
            chest: "DRIFT",
            necklace: false,
            banner: Rgb::hex(0x0d1117),
        },
        OppId::Docker => Look {
            p: Palette {
                top: Rgb::hex(0x1d63ed),
                top_dark: Rgb::hex(0x0b3d91),
                trunks: Rgb::hex(0x0b3d91),
                glove: Rgb::hex(0xf2f4f8),
                glove_dark: Rgb::hex(0xb8c2d0),
                cuff: Rgb::hex(0x1d63ed),
                hair: Rgb::hex(0xf2f4f8),
                skin: Rgb::hex(0xd9a57a),
                skin_dark: Rgb::hex(0xb3845c),
                ..BASE
            },
            torso_dw: 12,
            head_r: 7.5,
            hair: Hair::Cap,
            logo: Logo::Whale,
            chest: "LATEST",
            necklace: false,
            banner: Rgb::hex(0x0b3d91),
        },
        OppId::Snyk => Look {
            p: Palette {
                top: Rgb::hex(0x4c4a73),
                top_dark: Rgb::hex(0x2d2b4a),
                trunks: Rgb::hex(0x1c1b2e),
                glove: Rgb::hex(0xe06c9f),
                glove_dark: Rgb::hex(0x9a3f6a),
                hair: Rgb::hex(0x5c3b1e),
                ..BASE
            },
            torso_dw: -2,
            head_r: 6.0,
            hair: Hair::Curly,
            logo: Logo::Shield,
            chest: "CRIT!",
            necklace: false,
            banner: Rgb::hex(0x2d2b4a),
        },
        OppId::Bitnami => Look {
            p: Palette {
                top: Rgb::hex(0xcc092f),
                top_dark: Rgb::hex(0x7a0a1e),
                trunks: Rgb::hex(0x3a3a3a),
                glove: Rgb::hex(0x2b2b2b),
                glove_dark: Rgb::hex(0x101010),
                skin: Rgb::hex(0xf1c7a4),
                skin_dark: Rgb::hex(0xd3a07c),
                ..BASE
            },
            torso_dw: 4,
            head_r: 6.5,
            hair: Hair::Bald,
            logo: Logo::Circle,
            chest: "TRUST",
            necklace: false,
            banner: Rgb::hex(0x7a0a1e),
        },
        OppId::Devbox => Look {
            p: Palette {
                top: Rgb::hex(0x1aa37a),
                top_dark: Rgb::hex(0x0f6b50),
                trunks: Rgb::hex(0x0f3d2e),
                glove: Rgb::hex(0xffd166),
                glove_dark: Rgb::hex(0xc99a2e),
                hair: Rgb::hex(0x1b1b1f),
                ..BASE
            },
            torso_dw: -6,
            head_r: 6.0,
            hair: Hair::Spiky,
            logo: Logo::Cube,
            chest: "HOST",
            necklace: false,
            banner: Rgb::hex(0x0f6b50),
        },
        OppId::Flox => Look {
            p: Palette {
                top: Rgb::hex(0xf28c28),
                top_dark: Rgb::hex(0xb55d0e),
                trunks: Rgb::hex(0x7a3b0c),
                glove: Rgb::hex(0xe63946),
                glove_dark: Rgb::hex(0x9b1f2a),
                hair: Rgb::hex(0x7a4b1e),
                ..BASE
            },
            torso_dw: 0,
            head_r: 6.5,
            hair: Hair::Buzz,
            logo: Logo::Flake,
            chest: "REPRO*",
            necklace: false,
            banner: Rgb::hex(0xb55d0e),
        },
        OppId::Chainguard => Look {
            p: BASE,
            torso_dw: 0,
            head_r: 6.5,
            hair: Hair::Buzz,
            logo: Logo::Chain,
            chest: "0 CVE",
            necklace: true,
            banner: Rgb::hex(0x4b33b5),
        },
        OppId::Nix => Look {
            p: Palette {
                top: Rgb::hex(0x5277c3),
                top_dark: Rgb::hex(0x3b5998),
                trunks: Rgb::hex(0x1e3a6d),
                glove: Rgb::hex(0x7ebae4),
                glove_dark: Rgb::hex(0x4a86b8),
                hair: Rgb::hex(0xd8d8e0),
                skin: Rgb::hex(0xf3d9c2),
                skin_dark: Rgb::hex(0xd6b394),
                ..BASE
            },
            torso_dw: 2,
            head_r: 6.5,
            hair: Hair::Wizard,
            logo: Logo::Flake,
            chest: "PURE",
            necklace: false,
            banner: Rgb::hex(0x3b5998),
        },
    }
}

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
    left: Glove(3, 32, 1.0),
    right: Glove(41, 32, 1.0),
    prop: None,
};

/// Opponent pose for the current sim state. Rig box is 44 wide, 56 tall.
pub fn opp_pose(fi: &Fighter, tick: u64, id: OppId) -> Pose {
    use Action::*;
    let bob = ((tick / 8) % 2) as i32; // idle breathing
    if fi.hit_flash > 0 && !matches!(fi.state, State::Stunned(_)) {
        return Pose { body_dx: 2, head_dx: 4, head_dy: 1, face: Face::Hurt, left: Glove(6, 30, 1.0), right: Glove(40, 28, 1.0), ..GUARD };
    }
    match fi.state {
        State::Idle => Pose { body_dy: bob, head_dy: bob, left: Glove(3, 32 + bob, 1.0), right: Glove(41, 32 + bob, 1.0), ..GUARD },
        State::Startup(Jab, _) => Pose { face: Face::Focus, right: Glove(41, 8, 1.0), ..GUARD },
        State::Recovery(Jab, _) => Pose { face: Face::Focus, body_dx: -1, right: Glove(27, 30, 1.9), ..GUARD },
        State::Startup(Hook, _) => Pose { body_dx: 4, head_dx: 5, face: Face::Focus, left: Glove(-6, 16, 1.1), right: Glove(38, 24, 1.0), ..GUARD },
        State::Recovery(Hook, _) => Pose { body_dx: -2, head_dx: -3, face: Face::Focus, left: Glove(30, 12, 1.7), right: Glove(36, 26, 1.0), ..GUARD },
        State::Startup(Uppercut, _) => Pose { body_dy: 5, head_dy: 7, face: Face::Focus, left: Glove(8, 44, 1.0), right: Glove(36, 46, 1.1), ..GUARD },
        State::Recovery(Uppercut, _) => Pose { body_dy: -2, head_dy: -3, face: Face::Focus, left: Glove(8, 30, 1.0), right: Glove(24, -4, 1.6), ..GUARD },
        State::Startup(Special, _) => match id {
            OppId::Chainguard => Pose { face: Face::Smug, left: Glove(10, 24, 1.0), right: Glove(38, 6, 1.0), prop: Some(Prop::Briefcase(31, 0)), ..GUARD },
            OppId::Snyk => Pose { face: Face::Smug, body_dx: -3, head_dx: -4, left: Glove(2, 26, 1.0), right: Glove(52, 20, 1.0), ..GUARD },
            _ => Pose { body_dy: 3, head_dy: 4, face: Face::Focus, left: Glove(4, 40, 1.0), right: Glove(40, 2, 1.1), ..GUARD },
        },
        State::Recovery(Special, _) => match id {
            OppId::Bitnami => Pose { face: Face::Smug, left: Glove(2, 20, 1.0), right: Glove(42, 20, 1.0), ..GUARD },
            _ => Pose { body_dy: -1, face: Face::Smug, left: Glove(4, 30, 1.0), right: Glove(22, 24, 2.8), ..GUARD },
        },
        State::Blocking(_) => Pose { head_dy: 1, face: Face::Focus, left: Glove(16, 7, 1.1), right: Glove(28, 7, 1.1), ..GUARD },
        State::Dodging(Side::Left, _) => Pose { body_dx: -6, head_dx: -9, left: Glove(0, 24, 1.0), right: Glove(32, 24, 1.0), ..GUARD },
        State::Dodging(Side::Right, _) => Pose { body_dx: 6, head_dx: 9, left: Glove(12, 24, 1.0), right: Glove(44, 24, 1.0), ..GUARD },
        State::Stunned(_) => {
            let sway = (((tick as f32) / 4.0).sin() * 3.0) as i32;
            Pose { body_dx: sway / 2, head_dx: sway, head_dy: 2, face: Face::Dizzy, left: Glove(4, 40, 1.0), right: Glove(40, 42, 1.0), ..GUARD }
        }
    }
}

/// Draw the big opponent rig with its top-left at (ox, oy).
pub fn draw_opponent(c: &mut Canvas, ox: i32, oy: i32, look: &Look, pose: &Pose, tick: u64) {
    let p = &look.p;
    let bx = ox + pose.body_dx;
    let by = oy + pose.body_dy;
    let o = p.outline;
    let dw = look.torso_dw;
    let half = dw / 2;

    // Legs and boots.
    for lx in [13 - half / 2, 24 + half / 2] {
        c.rect_o(bx + lx, by + 41, 7, 11, p.skin, o);
        c.rect_o(bx + lx - 1, by + 51, 9, 5, p.trunks, o);
        c.rect(bx + lx - 1, by + 51, 9, 1, p.top);
    }
    // Trunks with waistband.
    c.rect_o(bx + 11 - half, by + 33, 22 + dw, 9, p.trunks, o);
    c.rect(bx + 11 - half, by + 33, 22 + dw, 2, p.band);
    // Torso, shoulders, tank top.
    c.rect_o(bx + 10 - half, by + 17, 24 + dw, 17, p.skin, o);
    c.ellipse_o(bx + 11 - half, by + 20, 3.5, 3.0, p.skin, o);
    c.ellipse_o(bx + 33 + half, by + 20, 3.5, 3.0, p.skin, o);
    c.rect(bx + 13 - half, by + 19, 18 + dw, 15, p.top);
    c.rect(bx + 13 - half, by + 19, 1, 15, p.top_dark);
    c.rect(bx + 30 + half, by + 19, 1, 15, p.top_dark);
    c.rect(bx + 14 - half, by + 16, 3, 4, p.top);
    c.rect(bx + 27 + half, by + 16, 3, 4, p.top);
    // Chest text and logo.
    let w = Canvas::text_width(look.chest, 1);
    c.text(bx + 22 - w / 2, by + 20, look.chest, 1, p.logo);
    draw_logo(c, bx + 22, by + 30, look.logo, p.logo, p.top);
    if look.necklace {
        for x in 12..=32 {
            let d = (x - 22) as i32;
            let y = by + 17 + (d * d) / 30;
            let col = if (x / 2) % 2 == 0 { p.chain } else { p.chain_dark };
            c.rect(bx + x, y, 1, 2, col);
        }
    }
    // Neck.
    c.rect(bx + 20, by + 13, 5, 5, p.skin_dark);

    // Head.
    let hx = ox + pose.head_dx + 22;
    let hy = oy + pose.head_dy + 8;
    let r = look.head_r;
    if look.hair == Hair::Hoodie {
        c.ellipse_o(hx, hy + 2, r + 2.5, r + 2.0, p.top, o);
        c.ellipse(hx, hy + 2, r + 1.0, r + 0.5, p.top_dark);
    }
    c.ellipse_o(hx, hy, r, r, p.skin, o);
    c.ellipse(hx - r as i32 - 1, hy, 1.5, 2.0, p.skin_dark);
    c.ellipse(hx + r as i32 + 1, hy, 1.5, 2.0, p.skin_dark);
    draw_hair(c, hx, hy, r, look.hair, p);
    face(c, hx, hy, pose.face, o, p.band, tick);
    if look.hair == Hair::Wizard {
        // Beard over the mouth, drawn after the face.
        c.ellipse_o(hx, hy + r as i32 + 1, r - 1.5, 4.0, p.hair, o);
        c.ellipse_o(hx, hy + r as i32 + 4, r - 3.0, 4.0, p.hair, o);
        c.ellipse(hx, hy + r as i32 + 2, r - 2.5, 3.0, p.hair);
    }

    // Prop, behind the glove that holds it.
    if let Some(Prop::Briefcase(px, py)) = pose.prop {
        briefcase(c, ox + px, oy + py, p);
    }

    // Arms and gloves, drawn last so they sit in front of the body.
    let (lsx, lsy) = (bx + 9 - half, by + 22);
    let (rsx, rsy) = (bx + 35 + half, by + 22);
    arm(c, lsx, lsy, ox + pose.left.0 - half, oy + pose.left.1, pose.left.2, p);
    arm(c, rsx, rsy, ox + pose.right.0 + half, oy + pose.right.1, pose.right.2, p);
}

fn draw_hair(c: &mut Canvas, hx: i32, hy: i32, r: f32, hair: Hair, p: &Palette) {
    match hair {
        Hair::Buzz => c.ellipse(hx, hy - 4, r - 0.5, 2.5, p.hair),
        Hair::Bald => {
            c.ellipse(hx - 2, hy - 4, 1.5, 1.0, p.skin.mix(Rgb::hex(0xffffff), 0.5));
        }
        Hair::Cap => {
            c.ellipse(hx, hy - 4, r + 0.5, 3.0, p.hair);
            c.rect(hx - r as i32 - 3, hy - 2, (r as i32) * 2 + 7, 2, p.top);
            c.rect(hx - r as i32 - 1, hy - 3, (r as i32) * 2 + 3, 1, p.top);
        }
        Hair::Hoodie => {}
        Hair::Wizard => {
            c.ellipse(hx, hy - 4, r - 0.5, 2.5, p.hair);
            c.rect(hx - r as i32 - 1, hy - 2, 3, 12, p.hair);
            c.rect(hx + r as i32 - 1, hy - 2, 3, 12, p.hair);
        }
        Hair::Curly => {
            for (dx, dy) in [(-5, -4), (-2, -6), (2, -6), (5, -4), (-6, -1), (6, -1)] {
                c.ellipse(hx + dx, hy + dy, 2.3, 2.0, p.hair);
            }
        }
        Hair::Spiky => {
            for (i, dx) in [-5, -2, 1, 4].iter().enumerate() {
                let h = if i % 2 == 0 { 5 } else { 4 };
                c.rect(hx + dx, hy - 5 - h, 2, h + 2, p.hair);
            }
            c.ellipse(hx, hy - 4, r - 0.5, 2.0, p.hair);
        }
    }
}

/// Brand logos, drawn in `ink` on a `bg` coloured surface, centred at (cx, cy).
pub fn draw_logo(c: &mut Canvas, cx: i32, cy: i32, logo: Logo, ink: Rgb, bg: Rgb) {
    match logo {
        Logo::Chain => {
            for dx in [-3, 3] {
                c.ellipse(cx + dx, cy, 3.6, 2.4, ink);
                c.ellipse(cx + dx, cy, 2.0, 1.0, bg);
            }
        }
        Logo::Whale => {
            c.ellipse(cx, cy + 1, 6.0, 2.5, ink);
            c.rect(cx + 5, cy - 2, 2, 2, ink);
            c.rect(cx + 6, cy - 3, 2, 1, ink);
            for dx in [-4, -1, 2] {
                c.rect(cx + dx, cy - 3, 2, 2, ink);
            }
            c.rect(cx - 1, cy - 5, 2, 2, ink);
            c.rect(cx - 4, cy + 1, 1, 1, bg);
        }
        Logo::Cat => {
            c.ellipse(cx, cy + 1, 4.0, 3.5, ink);
            c.rect(cx - 4, cy - 4, 2, 3, ink);
            c.rect(cx + 2, cy - 4, 2, 3, ink);
            c.rect(cx - 2, cy, 1, 1, bg);
            c.rect(cx + 1, cy, 1, 1, bg);
            c.rect(cx + 4, cy + 3, 3, 1, ink);
            c.rect(cx + 6, cy + 1, 1, 3, ink);
        }
        Logo::Shield => {
            c.rect(cx - 4, cy - 4, 9, 5, ink);
            c.ellipse(cx, cy + 1, 4.5, 3.5, ink);
            c.text(cx - 1, cy - 3, "!", 1, bg);
        }
        Logo::Circle => {
            c.ellipse(cx, cy, 4.5, 4.0, ink);
            c.text(cx - 1, cy - 2, "B", 1, bg);
        }
        Logo::Cube => {
            c.rect(cx - 5, cy - 2, 7, 6, ink);
            c.rect(cx - 4, cy - 1, 5, 4, bg);
            c.rect(cx - 2, cy - 5, 7, 6, ink);
            c.rect(cx - 1, cy - 4, 5, 4, bg);
        }
        Logo::Flake => {
            for k in 0..3 {
                let a = std::f32::consts::PI / 3.0 * k as f32;
                let (dx, dy) = ((a.cos() * 5.0).round() as i32, (a.sin() * 5.0).round() as i32);
                c.line(cx - dx, cy - dy, cx + dx, cy + dy, 1.2, ink);
            }
            c.rect(cx, cy, 1, 1, bg);
        }
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
    c.rect_o(bx + 5, by + 21, 14, 5, p.trunks, o);
    c.rect(bx + 5, by + 21, 14, 1, p.band);
    c.rect_o(bx + 5, by + 11, 14, 11, p.skin, o);
    c.ellipse_o(bx + 6, by + 13, 3.0, 2.5, p.skin, o);
    c.ellipse_o(bx + 18, by + 13, 3.0, 2.5, p.skin, o);
    c.rect(bx + 11, by + 11, 2, 11, p.skin_dark);
    let hx = ox + pose.head_dx + 12;
    let hy = oy + pose.head_dy + 6;
    c.rect(hx - 2, hy + 4, 4, 3, p.skin_dark);
    c.ellipse_o(hx, hy, 5.5, 5.5, p.hair, o);
    c.ellipse(hx - 1, hy - 2, 3.0, 2.0, p.hair.mix(Rgb::hex(0xffffff), 0.12));
    arm(c, bx + 6, by + 13, ox + pose.left.0, oy + pose.left.1, pose.left.2 * 0.75, p);
    arm(c, bx + 18, by + 13, ox + pose.right.0, oy + pose.right.1, pose.right.2 * 0.75, p);
}

