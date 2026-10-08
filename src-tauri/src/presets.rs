//! Programas populares de diseño, arte digital y 3D listos para añadir.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Preset {
    pub name: &'static str,
    pub exe: &'static str,
    pub shortcut: &'static str,
}

const fn preset(name: &'static str, exe: &'static str) -> Preset {
    Preset { name, exe, shortcut: "Ctrl+S" }
}

pub const PRESETS: &[Preset] = &[
    preset("Blender", "blender.exe"),
    preset("Krita", "krita.exe"),
    preset("Adobe Photoshop", "photoshop.exe"),
    preset("Adobe Illustrator", "illustrator.exe"),
    preset("Adobe After Effects", "afterfx.exe"),
    preset("Adobe Substance 3D Painter", "adobe substance 3d painter.exe"),
    preset("Clip Studio Paint", "clipstudiopaint.exe"),
    preset("Aseprite", "aseprite.exe"),
    preset("GIMP 2.10", "gimp-2.10.exe"),
    preset("GIMP 3", "gimp-3.0.exe"),
    preset("Inkscape", "inkscape.exe"),
    preset("Affinity Photo 2", "photo.exe"),
    preset("Affinity Designer 2", "designer.exe"),
    preset("Paint Tool SAI 2", "sai2.exe"),
    preset("MediBang Paint Pro", "medibangpaintpro.exe"),
    preset("FireAlpaca", "firealpaca.exe"),
    preset("Paint.NET", "paintdotnet.exe"),
    preset("OpenToonz", "opentoonz.exe"),
    preset("ZBrush", "zbrush.exe"),
    preset("Autodesk Maya", "maya.exe"),
    preset("Autodesk 3ds Max", "3dsmax.exe"),
    preset("Cinema 4D", "cinema 4d.exe"),
    preset("Houdini", "houdinifx.exe"),
];
