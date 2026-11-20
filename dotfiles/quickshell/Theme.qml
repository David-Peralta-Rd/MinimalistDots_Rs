pragma Singleton
import QtQuick
import "." as Local

QtObject {
    property Local.Palette palette: Local.Palette {}

    // Colores base desde la paleta generada.
    readonly property color fondo: palette.fondo
    readonly property color superficie: palette.superficie
    readonly property color seleccion: palette.seleccion
    readonly property color bordeInactivo: palette.bordeInactivo
    readonly property color bordeActivo: palette.bordeActivo
    readonly property color texto: palette.texto
    readonly property color textoSuave: palette.textoSuave
    readonly property color acentoAzul: palette.acentoAzul
    readonly property color acentoVerde: palette.acentoVerde
    readonly property color acentoRojo: palette.acentoRojo
    readonly property color acentoAmarillo: palette.acentoAmarillo
    readonly property color acentoMagenta: palette.acentoMagenta
    readonly property color acentoCian: palette.acentoCian
    readonly property color hover: palette.hover
    readonly property color tarjeta: palette.tarjeta
    readonly property color textoPausado: palette.textoPausado
    readonly property color contrasteClaro: palette.contrasteClaro
    readonly property color contrasteOscuro: palette.contrasteOscuro
    readonly property color overlay: palette.overlay

    // Geometría compartida.
    readonly property int radiusPill: 20
    readonly property int radiusCard: 12
    readonly property int borderWidth: 1
    readonly property int spacingSmall: 4
    readonly property int spacingMedium: 8

    // Tipografía compartida.
    readonly property string fontFamily: "monospace"
    readonly property int fontSizeSmall: 12
    readonly property int fontSizeNormal: 14
    readonly property int fontSizeLarge: 18
    readonly property int fontWeightNormal: Font.Medium
    readonly property int fontWeightBold: Font.Bold
}