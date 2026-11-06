//! Curated Bilingual Literary & Typographic Anthology (`anthology.rs`).
//!
//! Provides annotated reference specimens spanning Russian poetic prose, classical
//! European typography treatises, and rhythmic experiments for comparative calibration.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecimenTradition {
    RussianAcmeistProse,
    ClassicalBookTypography,
    RhythmicCadenceStudy,
    MinimalistLandscape,
    PhilosophicalHorology,
    BilingualEquilibrium,
}

#[derive(Debug, Clone, Copy)]
pub struct AnthologySpecimen {
    pub id: &'static str,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub year: &'static str,
    pub tradition: SpecimenTradition,
    pub recommended_canon: &'static str,
    pub text: &'static str,
    pub curatorial_commentary: &'static str,
}

pub const ANTHOLOGY_SPECIMENS: [AnthologySpecimen; 8] = [
    AnthologySpecimen {
        id: "architecture-of-silence",
        title: "Архитектура тишины и серого поля",
        subtitle: "Типографическое эссе · Антиква и воздух",
        year: "2026",
        tradition: SpecimenTradition::ClassicalBookTypography,
        recommended_canon: "VanDeGraafMedieval",
        text: "Тишина имеет собственный вес. Когда взгляд скользит по широкому полю тёплой архивной бумаги, каждая тонкая засечка удерживает равновесие между светом и тенью. Хороший шрифт не кричит о себе и не требует внимания. Он становится прозрачным сосудом для голоса, сохраняя дыхание между словами, паузу после точки и мерный пульс согласных. Слушайте эту музыку.",
        curatorial_commentary: "Образец классического контрапункта: открытие из четырёх слов (Staccato) сменяется широким 21-словным периодом (Legato) и завершается трёхсловной кодой.",
    },
    AnthologySpecimen {
        id: "mandelstam-stone",
        title: "Камень и акустика свода",
        subtitle: "Поэтическая проза · Акмеистический этюд",
        year: "1913 / Редакция",
        tradition: SpecimenTradition::RussianAcmeistProse,
        recommended_canon: "TschicholdGolden",
        text: "Кружевом, камнем и воздухом держится высокий готический свод. Мы не летаем, мы поднимаемся только на те башни, какие сами можем построить из твёрдого слова. В каждом слоге живёт точная мера тяжести, как в отвесе каменщика или натяжении тетивы. Уберите одно лишнее звено — и вся арка зазвучит иначе, чище и строже.",
        curatorial_commentary: "Высокая концентрация взрывных («к», «т», «б», «п») и сонорных («м», «н», «р», «л») формирует плотный, почти скульптурный рельеф строки.",
    },
    AnthologySpecimen {
        id: "provost-rhythm",
        title: "Урок ритмической каденции",
        subtitle: "Наблюдение о длине фразы · По мотиву Г. Провоста",
        year: "1985 / Адаптация",
        tradition: SpecimenTradition::RhythmicCadenceStudy,
        recommended_canon: "BringhurstEditorial",
        text: "В этой фразе пять слов. Вот ещё пять простых слов. Предложения такой длины вполне хороши. Но несколько подряд быстро утомляют. Слух начинает уставать от монотонности. А теперь послушайте, что происходит, когда писатель меняет шаг, отпускает короткую пружину и позволяет предложению развернуться во всю ширину страницы, набирая силу, плавность и глубокое дыхание, прежде чем снова замкнуть мысль коротким ударом. Видите разницу?",
        curatorial_commentary: "Пять намеренно монотонных пятисловных фраз создают упругое ожидание, которое разрешается 38-словным периодом Legato и двухсловным финалом.",
    },
    AnthologySpecimen {
        id: "northern-light",
        title: "Геометрия северного света",
        subtitle: "Минималистичная зарисовка · Свет и вода",
        year: "2026",
        tradition: SpecimenTradition::MinimalistLandscape,
        recommended_canon: "SwissModular",
        text: "Белый туман медленно сходит в холодную воду залива. Сосны стоят неподвижно. Ни ветра, ни шороха волны у песчаной косы, только бледное серебро утреннего неба и тонкая линия горизонта, проведённая сухой иглой по влажному листу.",
        curatorial_commentary: "Преобладание открытых гласных и шелестящих фрикативных («с», «ш», «в», «х») создаёт светлую оптическую плотность полосы и эффект водной глади.",
    },
    AnthologySpecimen {
        id: "horology-treatise",
        title: "Механика анкера и время строки",
        subtitle: "Философская миниатюра · Часовое дело и набор",
        year: "2026",
        tradition: SpecimenTradition::PhilosophicalHorology,
        recommended_canon: "TschicholdGolden",
        text: "Точность живёт в малом. Тонкий зубчик латунного колеса, сухой щелчок анкерной вилки, спокойное качание маятника в застеклённом футляре — всё это сродни искусству книжного набора. Если шпация между словами слишком широка, взгляд проваливается в пустоту; если строки сжаты слишком тесно, страница теряет воздух и перестаёт дышать.",
        curatorial_commentary: "Демонстрация синтаксического параллелизма через точку с запятой, уравновешивающего две зеркальные условные клаузы («Если...; если...»).",
    },
    AnthologySpecimen {
        id: "venetian-press",
        title: "Венецианская печатня Альда Мануция",
        subtitle: "Исторический очерк · Рождение курсива и октаво",
        year: "1501 / Монография",
        tradition: SpecimenTradition::ClassicalBookTypography,
        recommended_canon: "VanDeGraafMedieval",
        text: "В самом начале шестнадцатого столетия в мастерской у канала Риальто книга впервые обрела соразмерность человеческой ладони. Гравёр Франческо Гриффо вырезал пуансоны, в которых наклон гуманистического письма соединился со строгостью римских капителей. С тех пор чтение перестало быть неподвижным трудом в монастырском скриптории и превратилось в свободную беседу наедине со страницей.",
        curatorial_commentary: "Плавное повествовательное дыхание с высокой долей полногласных и сонорных корней, идеально раскрывающее пластику гарнитуры Cormorant Garamond.",
    },
    AnthologySpecimen {
        id: "winter-hermitage",
        title: "Снег на граните набережной",
        subtitle: "Поэтический этюд · Пауза и перспектива",
        year: "2026",
        tradition: SpecimenTradition::RussianAcmeistProse,
        recommended_canon: "BringhurstEditorial",
        text: "Первый снег ложится на тёмный гранит парапета без единого звука. Чёрная вода канала кажется ещё глубже в раме побелевших карнизов и чугунных решёток. Как странно, что несколько белых штрихов способны за одну ночь переписать всю архитектуру города, превратив тяжёлый камень в лёгкий карандашный набросок на полях зимы.",
        curatorial_commentary: "Богатая аллитерация на «гр», «кр», «чр» в первой половине отрывка смягчается мягкими палатальными гласными в финале.",
    },
    AnthologySpecimen {
        id: "bringhurst-canon-en",
        title: "Typography Honors the Text",
        subtitle: "Bilingual Specimen · After Robert Bringhurst",
        year: "1992 / Study",
        tradition: SpecimenTradition::BilingualEquilibrium,
        recommended_canon: "BringhurstEditorial",
        text: "Typography exists to honor content. Like a musical performance, a well-typeset page turns silent symbols into a living cadence of thought and breath. Letters have their own quiet dignity when spaced with care, set to a comfortable measure, and printed upon paper that welcomes the eye.",
        curatorial_commentary: "Англоязычный эталон для проверки латинских таблиц кернинга (`Ty`, `To`, `Av`, `Wa`) и эвристики немого конечного `-e` в подсчёте слогов.",
    },
];

/// Looks up an anthology specimen by its identifier.
pub fn find_specimen_by_id(id: &str) -> Option<&'static AnthologySpecimen> {
    ANTHOLOGY_SPECIMENS.iter().find(|s| s.id == id)
}
