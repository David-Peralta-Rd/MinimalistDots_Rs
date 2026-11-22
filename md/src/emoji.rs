//! Subcomando `emoji`: selector de emojis que copia la elección al portapapeles.

use std::collections::HashMap;

use md_core::MenuEntry;

use crate::i18n::Texts;
use crate::menu::{MenuResult, run_entries};
use crate::util;

/// Lista curada de emojis (símbodo, nombre). El nombre sirve para filtrar.
const EMOJIS: &[(&str, &str)] = &[
    ("😀", "cara sonriente"),
    ("😃", "cara sonriente ojos grandes"),
    ("😄", "cara sonriendo ojos sonrientes"),
    ("😁", "cara radiante"),
    ("😆", "cara riendo"),
    ("😅", "cara sonriente sudor"),
    ("🤣", "revolcándose de risa"),
    ("😂", "lágrimas de alegría"),
    ("🙂", "cara ligeramente sonriente"),
    ("😉", "guiño"),
    ("😊", "sonrisa tímida"),
    ("😇", "ángel"),
    ("🥰", "cara con corazones"),
    ("😍", "corazones en los ojos"),
    ("🤩", "estrella en los ojos"),
    ("😘", "beso volado"),
    ("😗", "beso"),
    ("😚", "beso con ojos cerrados"),
    ("😙", "beso sonriente"),
    ("😋", "saboreando"),
    ("😛", "sacando lengua"),
    ("😜", "lengua guiño"),
    ("🤪", "cara loca"),
    ("😝", "lengua ojos cerrados"),
    ("🤑", "cara dinero"),
    ("🤗", "abrazo"),
    ("🤭", "mano en la boca"),
    ("🤫", "silencio"),
    ("🤔", "pensando"),
    ("🤐", "cremallera"),
    ("🤨", "ceja levantada"),
    ("😐", "neutral"),
    ("😑", "expresión plana"),
    ("😶", "sin boca"),
    ("😏", "sonrisa maliciosa"),
    ("😒", "descontento"),
    ("🙄", "ojos en blanco"),
    ("😬", "cara forzada"),
    ("🤥", "mentiroso"),
    ("😌", "aliviado"),
    ("😔", "pensativo"),
    ("😪", "soñoliento"),
    ("🤤", "baboso"),
    ("😴", "durmiendo"),
    ("😷", "mascarilla"),
    ("🤒", "termómetro"),
    ("🤕", "venda"),
    ("🤢", "náuseas"),
    ("🤮", "vomitando"),
    ("🤧", "estornudo"),
    ("🥵", "calor"),
    ("🥶", "frío"),
    ("🥴", "mareado"),
    ("😵", "aturdido"),
    ("🤯", "cabeza explotando"),
    ("🤠", "vaquero"),
    ("🥳", "fiesta"),
    ("😎", "gafas de sol"),
    ("🤓", "nerd"),
    ("🧐", "monóculo"),
    ("😕", "confundido"),
    ("😟", "preocupado"),
    ("🙁", "triste"),
    ("☹️", "fruncido"),
    ("😮", "sorpresa"),
    ("😯", "sorprendido"),
    ("😲", "asombrado"),
    ("😳", "sonrojado"),
    ("🥺", "súplica"),
    ("😦", "fruncido abierto"),
    ("😧", "angustiado"),
    ("😨", "miedoso"),
    ("😰", "sudor ansiedad"),
    ("😥", "decepcionado aliviado"),
    ("😢", "llorando"),
    ("😭", "llanto"),
    ("😱", "grito"),
    ("😖", "frustrado"),
    ("😣", "perseverante"),
    ("😞", "decepcionado"),
    ("😓", "sudor frío"),
    ("😩", "cansado"),
    ("😫", "agotado"),
    ("🥱", "bostezo"),
    ("😤", "humo narices"),
    ("😡", "enfadado"),
    ("😠", "enojado"),
    ("🤬", "insultando"),
    ("😈", "diablo sonriente"),
    ("👿", "diablo enfadado"),
    ("💀", "calavera"),
    ("☠️", "calavera huesos"),
    ("💩", "caca"),
    ("🤡", "payaso"),
    ("👹", "ogro"),
    ("👺", "duende"),
    ("👻", "fantasma"),
    ("👽", "alien"),
    ("👾", "monstruo espacio"),
    ("🤖", "robot"),
    ("😺", "gato sonriente"),
    ("😸", "gato ojos sonrientes"),
    ("😹", "gato lágrimas alegría"),
    ("😻", "gato corazones"),
    ("😼", "gato sonrisa maliciosa"),
    ("😽", "gato beso"),
    ("🙀", "gato sorprendido"),
    ("😿", "gato llorando"),
    ("😾", "gato enfadado"),
    ("🙈", "no ver"),
    ("🙉", "no oír"),
    ("🙊", "no hablar"),
    ("💋", "beso labios"),
    ("💌", "carta amor"),
    ("💘", "flecha cupido"),
    ("💝", "corazón lazo"),
    ("💖", "corazón brillante"),
    ("💗", "corazón creciendo"),
    ("💓", "corazón latiendo"),
    ("💞", "corazones girando"),
    ("💕", "dos corazones"),
    ("💟", "decoración corazón"),
    ("❣️", "corazón exclamación"),
    ("💔", "corazón roto"),
    ("❤️", "corazón rojo"),
    ("🧡", "corazón naranja"),
    ("💛", "corazón amarillo"),
    ("💚", "corazón verde"),
    ("💙", "corazón azul"),
    ("💜", "corazón morado"),
    ("🖤", "corazón negro"),
    ("🤍", "corazón blanco"),
    ("🤎", "corazón marrón"),
    ("💯", "cien"),
    ("💢", "símbodo enfado"),
    ("💥", "colisión"),
    ("💫", "mareado"),
    ("💦", "salpicadura"),
    ("💨", "dash"),
    ("🕳️", "agujero"),
    ("💣", "bomba"),
    ("💬", "bocadillo"),
    ("👁️‍🗨️", "ojo en bocadillo"),
    ("🗨️", "bocadillo izquierda"),
    ("🗯️", "bocadillo enfado"),
    ("💭", "pensamiento"),
    (" ", "espacio"),
    ("🍕", "pizza"),
    ("🍔", "hamburger"),
    ("🍟", "fries"),
    ("🌭", "hot dog"),
    ("🍿", "popcorn"),
    ("🧂", "salt"),
    ("🥚", "egg"),
    ("🍳", "cooking"),
    ("🧇", "waffle"),
    ("🥞", "pancakes"),
    ("🧈", "butter"),
    ("🍞", "bread"),
    ("🥐", "croissant"),
    ("🥖", "baguette"),
    ("🥨", "pretzel"),
    ("🧀", "cheese"),
    ("🥗", "salad"),
    ("🥙", "stuffed flatbread"),
    ("🧆", "falafel"),
    ("🌮", "taco"),
    ("🌯", "burrito"),
    ("🥫", "canned food"),
    ("🍝", "spaghetti"),
    ("🍜", "ramen"),
    ("🍲", "pot of food"),
    ("🍛", "curry rice"),
    ("🍣", "sushi"),
    ("🍱", "bento"),
    ("🥟", "dumpling"),
    ("🦪", "oyster"),
    ("🍤", "fried shrimp"),
    ("🍙", "rice ball"),
    ("🍚", "cooked rice"),
    ("🍘", "rice cracker"),
    ("🍥", "fish cake"),
    ("🥠", "fortune cookie"),
    ("🥮", "moon cake"),
    ("🍢", "oden"),
    ("🍡", "dango"),
    ("🍧", "shaved ice"),
    ("🍨", "ice cream"),
    ("🍦", "soft ice cream"),
    ("🥧", "pie"),
    ("🧁", "cupcake"),
    ("🍰", "shortcake"),
    ("🎂", "birthday cake"),
    ("🍮", "custard"),
    ("🍭", "lollipop"),
    ("🍬", "candy"),
    ("🍫", "chocolate"),
    ("🍩", "doughnut"),
    ("🍪", "cookie"),
    ("🌰", "chestnut"),
    ("🥜", "peanuts"),
    ("🍯", "honey pot"),
    ("🥛", "milk"),
    ("🍼", "baby bottle"),
    ("🫖", "teapot"),
    ("☕", "hot beverage"),
    ("🍵", "tea"),
    ("🧃", "beverage box"),
    ("🥤", "cup with straw"),
    ("🧋", "bubble tea"),
    ("🍶", "sake"),
    ("🍺", "beer"),
    ("🍻", "clinking beers"),
    ("🥂", "clinking glasses"),
    ("🍷", "wine"),
    ("🍸", "cocktail"),
    ("🍹", "tropical drink"),
    ("🍾", "bottle popping"),
    ("🧊", "ice"),
    ("🥄", "spoon"),
    ("🍴", "fork and knife"),
    ("🍽️", "fork and knife plate"),
    ("🥢", "chopsticks"),
    ("🧺", "basket"),
    ("🧼", "soap"),
    ("🧽", "sponge"),
    ("🧻", "roll of paper"),
    ("🪒", "razor"),
    ("🪮", "hair pick"),
    ("🪥", "toothbrush"),
    ("🧏", "deaf person"),
    ("🧍", "standing person"),
    ("🧎", "kneeling person"),
    ("🧏‍♂️", "deaf man"),
    ("🧏‍♀️", "deaf woman"),
    ("🧍‍♂️", "standing man"),
    ("🧍‍♀️", "standing woman"),
    ("🧎‍♂️", "kneeling man"),
    ("🧎‍♀️", "kneeling woman"),
    ("🧑‍🤝‍🧑", "people holding hands"),
    ("👩‍❤️‍👨", "couple with heart"),
    ("👩‍❤️‍👩", "couple with heart ww"),
    ("👨‍❤️‍👨", "couple with heart mm"),
    ("👩‍❤️‍💋‍👨", "kiss"),
    ("👩‍❤️‍💋‍👩", "kiss ww"),
    ("👨‍❤️‍💋‍👨", "kiss mm"),
    ("👨‍👩‍👦", "family"),
    ("👨‍👩‍👧", "family"),
    ("👨‍👩‍👧‍👦", "family"),
    ("👨‍👩‍👦‍👦", "family"),
    ("👨‍👩‍👧‍👧", "family"),
    ("👩‍👩‍👦", "family"),
    ("👩‍👩‍👧", "family"),
    ("👨‍👨‍👦", "family"),
    ("👨‍👨‍👧", "family"),
    ("👩‍👦", "family"),
    ("👩‍👧", "family"),
    ("👨‍👦", "family"),
    ("👨‍👧", "family"),
    ("🧓", "older adult"),
    ("👴", "old man"),
    ("👵", "old woman"),
    ("👤", "bust in silhouette"),
    ("👥", "busts in silhouette"),
    ("🫂", "people hugging"),
    ("🗣️", "speaking head"),
    ("👤", "bust"),
    ("👥", "busts"),
];

/// Carga nombres localizados desde la variable de idioma `escritorio.emoji_nombres`.
/// Formato esperado: una línea por emoji, `emoji nombre` (el nombre puede contener espacios).
fn localized_names() -> HashMap<String, String> {
    let texts = Texts::load();
    let raw = texts.t("escritorio.emoji_nombres");
    // Si no hay traducción, `t` devuelve la propia clave: la ignoramos.
    if raw == "escritorio.emoji_nombres" || raw.trim().is_empty() {
        return HashMap::new();
    }
    let mut map = HashMap::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // Separa por el primer espacio: emoji + resto (nombre).
        if let Some((emoji, name)) = line.split_once(char::is_whitespace) {
            let emoji = emoji.trim();
            let name = name.trim();
            if !emoji.is_empty() && !name.is_empty() {
                map.insert(emoji.to_owned(), name.to_owned());
            }
        }
    }
    map
}

/// Ejecuta el selector de emojis.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let texts = Texts::load();
    let prompt = texts.t("escritorio.emoji_prompt");
    let prompt = if prompt == "escritorio.emoji_prompt" || prompt.trim().is_empty() {
        "😀".to_owned()
    } else {
        prompt
    };

    let names = localized_names();

    let emojis: Vec<&str> = EMOJIS.iter().map(|(e, _)| *e).collect();
    let entries: Vec<MenuEntry> = EMOJIS
        .iter()
        .map(|(e, fallback)| {
            let name = names.get(*e).cloned().unwrap_or_else(|| (*fallback).to_owned());
            MenuEntry::new(format!("{e}  {name}"))
        })
        .collect();

    match run_entries(&entries, &prompt, false, false, false, None, None, None, None)? {
        MenuResult::Selected(index) => {
            if let Some(emoji) = emojis.get(index) {
                util::copy_to_clipboard(emoji)?;
            }
            Ok(())
        }
        MenuResult::Typed(query) => {
            // Si el usuario escribe un emoji directamente, lo copiamos tal cual.
            if !query.trim().is_empty() {
                util::copy_to_clipboard(&query)?;
            }
            Ok(())
        }
        MenuResult::Cancelled | MenuResult::Delete(_) | MenuResult::Wipe => Ok(()),
    }
}
