//! The words of the window in the eight languages of the repository (English, Spanish, German, French, Italian, Japanese, Chinese and Portuguese of Portugal).
//! Every sentence is one entry of the list below with its eight texts in that order, so a sentence cannot be added without all of them (the compiler counts them) and the
//! list of all sentences is made from the same entries (the tests walk it).

/// The languages of the window, in the order the texts of every sentence follow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    En,
    Es,
    De,
    Fr,
    It,
    Ja,
    Zh,
    Pt,
}

impl Lang {
    pub const ALL: [Lang; 8] = [
        Lang::En,
        Lang::Es,
        Lang::De,
        Lang::Fr,
        Lang::It,
        Lang::Ja,
        Lang::Zh,
        Lang::Pt,
    ];

    /// The two letters of the language (`en`, `es`, `de`, `fr`, `it`, `ja`, `zh`, `pt`).
    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Es => "es",
            Lang::De => "de",
            Lang::Fr => "fr",
            Lang::It => "it",
            Lang::Ja => "ja",
            Lang::Zh => "zh",
            Lang::Pt => "pt",
        }
    }

    /// The name of the language in the language itself.
    pub fn name(self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::Es => "Español",
            Lang::De => "Deutsch",
            Lang::Fr => "Français",
            Lang::It => "Italiano",
            Lang::Ja => "日本語",
            Lang::Zh => "中文",
            Lang::Pt => "Português",
        }
    }

    /// The language of a code such as `es`, `es_ES.UTF-8` or `pt-PT`.
    pub fn from_code(code: &str) -> Option<Lang> {
        let two: String = code
            .chars()
            .take(2)
            .collect::<String>()
            .to_ascii_lowercase();
        Lang::ALL.into_iter().find(|l| l.code() == two)
    }

    /// The language of the person from the variables of the system; English when it is none of the eight.
    pub fn from_env() -> Lang {
        for var in ["LC_ALL", "LC_MESSAGES", "LANG"] {
            if let Ok(v) = std::env::var(var) {
                if !v.is_empty() {
                    return Lang::from_code(&v).unwrap_or(Lang::En);
                }
            }
        }
        Lang::En
    }
}

/// Puts the values in the places `{0}`, `{1}`... of a text.
pub fn fill(template: &str, values: &[&str]) -> String {
    let mut out = template.to_string();
    for (k, v) in values.iter().enumerate() {
        out = out.replace(&format!("{{{k}}}"), v);
    }
    out
}

/// Makes the list of sentences, the list of all of them and the texts from one table.
macro_rules! messages {
    ($($name:ident => [$($text:expr),+ $(,)?]),+ $(,)?) => {
        /// Every word and sentence of the window.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Msg {
            $($name),+
        }

        impl Msg {
            /// Every sentence, once.
            pub const ALL: &[Msg] = &[$(Msg::$name),+];

            /// The words in the order English, Spanish, German, French, Italian, Japanese, Chinese, Portuguese.
            fn words(self) -> [&'static str; 8] {
                match self {
                    $(Msg::$name => [$($text),+]),+
                }
            }
        }
    };
}

messages! {
    TabTools => ["Tools", "Herramientas", "Werkzeuge", "Outils", "Utensili", "工具", "刀具", "Ferramentas"],
    TabCalc => ["Feeds and speeds", "Avances y velocidades", "Vorschübe und Drehzahlen", "Avances et vitesses", "Avanzamenti e velocità", "送りと回転数", "进给与转速", "Avanços e velocidades"],
    Language => ["Language", "Idioma", "Sprache", "Langue", "Lingua", "言語", "语言", "Idioma"],
    CribFile => ["Crib file (JSON)", "Fichero del almacén (JSON)", "Werkzeugdatei (JSON)", "Fichier de l'outillage (JSON)", "File del magazzino utensili (JSON)", "工具庫ファイル (JSON)", "刀具库文件 (JSON)", "Ficheiro do armazém (JSON)"],
    CsvFile => ["CSV table", "Tabla CSV", "CSV-Tabelle", "Tableau CSV", "Tabella CSV", "CSV表", "CSV 表格", "Tabela CSV"],
    Open => ["Open", "Abrir", "Öffnen", "Ouvrir", "Apri", "開く", "打开", "Abrir"],
    Save => ["Save", "Guardar", "Speichern", "Enregistrer", "Salva", "保存", "保存", "Guardar"],
    NewCrib => ["New crib", "Nuevo almacén", "Neuer Bestand", "Nouvel outillage", "Nuovo magazzino", "新規工具庫", "新建刀具库", "Novo armazém"],
    ImportCsv => ["Import CSV", "Importar CSV", "CSV importieren", "Importer le CSV", "Importa CSV", "CSVを読み込む", "导入 CSV", "Importar CSV"],
    ExportCsv => ["Export CSV", "Exportar CSV", "CSV exportieren", "Exporter en CSV", "Esporta CSV", "CSVに書き出す", "导出 CSV", "Exportar CSV"],
    Overwrite => ["Replace the file if it exists", "Reemplazar el fichero si existe", "Datei ersetzen, falls vorhanden", "Remplacer le fichier s'il existe", "Sostituisci il file se esiste", "ファイルが存在する場合は置き換える", "文件已存在时替换", "Substituir o ficheiro se existir"],
    Unsaved => ["Unsaved changes", "Cambios sin guardar", "Nicht gespeicherte Änderungen", "Modifications non enregistrées", "Modifiche non salvate", "未保存の変更", "有未保存的更改", "Alterações por guardar"],
    Search => ["Search", "Buscar", "Suchen", "Rechercher", "Cerca", "検索", "搜索", "Pesquisar"],
    KindLabel => ["Kind", "Tipo", "Art", "Type", "Tipo", "種類", "类型", "Tipo"],
    AllKinds => ["All kinds", "Todos los tipos", "Alle Arten", "Tous les types", "Tutti i tipi", "すべての種類", "所有类型", "Todos os tipos"],
    WearLabel => ["Wear", "Desgaste", "Verschleiß", "Usure", "Usura", "摩耗", "磨损", "Desgaste"],
    WearAll => ["All", "Todas", "Alle", "Toutes", "Tutte", "すべて", "全部", "Todas"],
    WearWorn => ["Worn or about to be", "Gastadas o a punto de estarlo", "Verschlissen oder bald", "Usées ou presque", "Usurate o quasi", "摩耗済みまたは間近", "已磨损或即将磨损", "Gastas ou quase"],
    WearReplaceOnly => ["To replace", "Para cambiar", "Zu ersetzen", "À remplacer", "Da sostituire", "交換が必要", "需要更换", "A substituir"],
    SortBy => ["Sort by", "Ordenar por", "Sortieren nach", "Trier par", "Ordina per", "並べ替え", "排序依据", "Ordenar por"],
    Descending => ["Descending", "Descendente", "Absteigend", "Décroissant", "Decrescente", "降順", "降序", "Decrescente"],
    Showing => ["{0} of {1} tools", "{0} de {1} herramientas", "{0} von {1} Werkzeugen", "{0} outils sur {1}", "{0} utensili su {1}", "{1} 本中 {0} 本", "共 {1} 把中的 {0} 把", "{0} de {1} ferramentas"],
    ColId => ["No.", "N.º", "Nr.", "N°", "N.", "番号", "编号", "N.º"],
    ColName => ["Name", "Nombre", "Name", "Nom", "Nome", "名前", "名称", "Nome"],
    ColDiameter => ["Diameter (mm)", "Diámetro (mm)", "Durchmesser (mm)", "Diamètre (mm)", "Diametro (mm)", "直径 (mm)", "直径 (mm)", "Diâmetro (mm)"],
    ColFlutes => ["Flutes", "Labios", "Schneiden", "Dents", "Taglienti", "刃数", "刃数", "Dentes"],
    ColAngle => ["Angle (°)", "Ángulo (°)", "Winkel (°)", "Angle (°)", "Angolo (°)", "角度 (°)", "角度 (°)", "Ângulo (°)"],
    ColHours => ["Hours", "Horas", "Stunden", "Heures", "Ore", "時間", "小时", "Horas"],
    ColHoles => ["Holes", "Agujeros", "Bohrungen", "Trous", "Fori", "穴数", "孔数", "Furos"],
    FTipAngle => ["Tip angle (°)", "Ángulo de la punta (°)", "Spitzenwinkel (°)", "Angle de la pointe (°)", "Angolo della punta (°)", "先端角 (°)", "刀尖角度 (°)", "Ângulo da ponta (°)"],
    FLength => ["Length (mm)", "Longitud (mm)", "Länge (mm)", "Longueur (mm)", "Lunghezza (mm)", "長さ (mm)", "长度 (mm)", "Comprimento (mm)"],
    FShank => ["Shank (mm)", "Mango (mm)", "Schaft (mm)", "Queue (mm)", "Codolo (mm)", "シャンク径 (mm)", "刀柄 (mm)", "Haste (mm)"],
    FMaterial => ["Tool material", "Material de la herramienta", "Werkzeugmaterial", "Matière de l'outil", "Materiale dell'utensile", "工具の材質", "刀具材料", "Material da ferramenta"],
    FMeters => ["Metres", "Metros", "Meter", "Mètres", "Metri", "メートル", "米数", "Metros"],
    FMaxHours => ["Hour limit (0 = none)", "Límite de horas (0 = ninguno)", "Stundenlimit (0 = keines)", "Limite d'heures (0 = aucune)", "Limite di ore (0 = nessuno)", "時間の上限 (0 = なし)", "小时上限 (0 = 无)", "Limite de horas (0 = nenhum)"],
    FMaxHoles => ["Hole limit (0 = none)", "Límite de agujeros (0 = ninguno)", "Bohrungslimit (0 = keines)", "Limite de trous (0 = aucune)", "Limite di fori (0 = nessuno)", "穴数の上限 (0 = なし)", "孔数上限 (0 = 无)", "Limite de furos (0 = nenhum)"],
    FormNew => ["New tool", "Herramienta nueva", "Neues Werkzeug", "Nouvel outil", "Nuovo utensile", "新しい工具", "新建刀具", "Nova ferramenta"],
    FormEdit => ["Editing tool {0}", "Editando la herramienta {0}", "Werkzeug {0} bearbeiten", "Modification de l'outil {0}", "Modifica dell'utensile {0}", "工具 {0} を編集中", "正在编辑刀具 {0}", "A editar a ferramenta {0}"],
    AddTool => ["Add tool", "Añadir herramienta", "Werkzeug hinzufügen", "Ajouter l'outil", "Aggiungi utensile", "工具を追加", "添加刀具", "Adicionar ferramenta"],
    SaveChanges => ["Apply changes", "Aplicar cambios", "Änderungen übernehmen", "Appliquer les modifications", "Applica le modifiche", "変更を適用", "应用更改", "Aplicar alterações"],
    ClearForm => ["Clear the form", "Vaciar el formulario", "Formular leeren", "Vider le formulaire", "Svuota il modulo", "フォームをクリア", "清空表单", "Limpar o formulário"],
    Edit => ["Edit", "Editar", "Bearbeiten", "Modifier", "Modifica", "編集", "编辑", "Editar"],
    Remove => ["Remove", "Quitar", "Entfernen", "Retirer", "Rimuovi", "削除", "移除", "Remover"],
    NoTools => ["The crib has no tools yet", "El almacén aún no tiene herramientas", "Der Bestand enthält noch keine Werkzeuge", "L'outillage ne contient pas encore d'outils", "Il magazzino non ha ancora utensili", "工具庫にはまだ工具がありません", "刀具库中还没有刀具", "O armazém ainda não tem ferramentas"],
    NoMatch => ["No tool matches the filters", "Ninguna herramienta cumple los filtros", "Kein Werkzeug entspricht den Filtern", "Aucun outil ne correspond aux filtres", "Nessun utensile corrisponde ai filtri", "フィルターに一致する工具はありません", "没有符合筛选条件的刀具", "Nenhuma ferramenta corresponde aos filtros"],
    UseTitle => ["Record the use of tool {0}", "Registrar el uso de la herramienta {0}", "Einsatz von Werkzeug {0} erfassen", "Enregistrer l'utilisation de l'outil {0}", "Registra l'uso dell'utensile {0}", "工具 {0} の使用を記録", "记录刀具 {0} 的使用", "Registar a utilização da ferramenta {0}"],
    AddUse => ["Add to the counters", "Sumar a los contadores", "Zu den Zählern addieren", "Ajouter aux compteurs", "Aggiungi ai contatori", "カウンターに加算", "累加到计数器", "Somar aos contadores"],
    WearGood => ["good ({0} %)", "bien ({0} %)", "gut ({0} %)", "bon ({0} %)", "buono ({0} %)", "良好 ({0} %)", "良好（{0} %）", "bom ({0} %)"],
    WearSoonText => ["replace soon ({0} %)", "cambiar pronto ({0} %)", "bald ersetzen ({0} %)", "à remplacer bientôt ({0} %)", "sostituire presto ({0} %)", "まもなく交換 ({0} %)", "即将更换（{0} %）", "substituir em breve ({0} %)"],
    WearReplaceText => ["REPLACE", "CAMBIAR", "ERSETZEN", "REMPLACER", "SOSTITUIRE", "交換", "更换", "SUBSTITUIR"],
    CalcIntro => ["Starting points to try on a scrap board, not a promise. The numbers come from tools.cfg; change them here to try other values.", "Puntos de partida para probar en una placa de desecho, no una garantía. Los números vienen de tools.cfg; cámbielos aquí para probar otros valores.", "Startwerte zum Ausprobieren auf Restmaterial, kein Versprechen. Die Zahlen stammen aus tools.cfg; hier lassen sie sich zum Testen ändern.", "Points de départ à essayer sur une chute, pas une promesse. Les valeurs viennent de tools.cfg ; modifiez-les ici pour en essayer d'autres.", "Punti di partenza da provare su uno scarto, non una promessa. I numeri vengono da tools.cfg; modificali qui per provarne altri.", "スクラップ板で試すための出発点であり、保証ではありません。数値は tools.cfg から読み込まれ、ここで変更して試せます。", "这些只是在废板上试切的起点，并非保证。数值来自 tools.cfg，可在此修改后试用。", "Pontos de partida para experimentar numa placa de sucata, não uma garantia. Os números vêm de tools.cfg; altere-os aqui para experimentar outros."],
    FromCrib => ["Take the values of a tool of the crib", "Tomar los valores de una herramienta del almacén", "Werte eines Werkzeugs aus dem Bestand übernehmen", "Prendre les valeurs d'un outil de l'outillage", "Prendi i valori di un utensile del magazzino", "工具庫の工具から値を取得", "取用刀具库中某把刀具的值", "Usar os valores de uma ferramenta do armazém"],
    WorkMaterial => ["Material to cut", "Material a cortar", "Zu bearbeitendes Material", "Matière à usiner", "Materiale da lavorare", "加工する材料", "被加工材料", "Material a cortar"],
    MaxRpm => ["Spindle, at most (rpm)", "Husillo, como máximo (rpm)", "Spindel, höchstens (U/min)", "Broche, au plus (tr/min)", "Mandrino, al massimo (giri/min)", "スピンドル最大 (rpm)", "主轴最高转速 (rpm)", "Fuso, no máximo (rpm)"],
    Result => ["Result", "Resultado", "Ergebnis", "Résultat", "Risultato", "結果", "结果", "Resultado"],
    ResRpm => ["Speed", "Velocidad", "Drehzahl", "Vitesse", "Velocità", "回転数", "转速", "Velocidade"],
    ResFeed => ["Feed", "Avance", "Vorschub", "Avance", "Avanzamento", "送り", "进给", "Avanço"],
    ResPlunge => ["Plunge", "Penetración", "Eintauchen", "Plongée", "Affondamento", "突っ込み", "下刀", "Penetração"],
    ResPass => ["Depth of a pass", "Profundidad de pasada", "Zustelltiefe", "Profondeur de passe", "Profondità di passata", "1パスの深さ", "单次切深", "Profundidade de passagem"],
    SpindleLimits => ["The spindle limits the speed", "El husillo limita la velocidad", "Die Spindel begrenzt die Drehzahl", "La broche limite la vitesse", "Il mandrino limita la velocità", "スピンドルが回転数を制限しています", "主轴限制了转速", "O fuso limita a velocidade"],
    PointNote => ["Only a starting point: what a V-bit or an engraver cuts with depends on the depth, try it on a scrap board", "Solo un punto de partida: con qué corta una fresa en V o un grabador depende de la profundidad, pruébelo en una placa de desecho", "Nur ein Startwert: womit ein V-Fräser oder Gravierstichel schneidet, hängt von der Tiefe ab; auf Restmaterial testen", "Seulement un point de départ : ce avec quoi une fraise en V ou un burin coupe dépend de la profondeur, essayez sur une chute", "Solo un punto di partenza: con che cosa taglia una fresa a V o un incisore dipende dalla profondità, provalo su uno scarto", "あくまで出発点です。Vビットや彫刻刀が切削する径は深さによって変わるため、スクラップ板で試してください", "仅为起点：V 形刀或雕刻刀的切削直径取决于切深，请先在废板上试切", "Apenas um ponto de partida: o que uma fresa em V ou um gravador corta depende da profundidade, experimente numa placa de sucata"],
    Parameters => ["Parameters of the calculation", "Parámetros del cálculo", "Parameter der Berechnung", "Paramètres du calcul", "Parametri del calcolo", "計算のパラメータ", "计算参数", "Parâmetros do cálculo"],
    ResetParams => ["Back to the values of tools.cfg", "Volver a los valores de tools.cfg", "Zurück zu den Werten aus tools.cfg", "Revenir aux valeurs de tools.cfg", "Torna ai valori di tools.cfg", "tools.cfg の値に戻す", "恢复为 tools.cfg 的值", "Voltar aos valores de tools.cfg"],
    ParSurface => ["Surface speed (m/min)", "Velocidad de corte (m/min)", "Schnittgeschwindigkeit (m/min)", "Vitesse de coupe (m/min)", "Velocità di taglio (m/min)", "切削速度 (m/min)", "切削速度 (m/min)", "Velocidade de corte (m/min)"],
    ParChip => ["Chip load (fraction of the diameter, per tooth)", "Avance por diente (fracción del diámetro)", "Zahnvorschub (Anteil des Durchmessers)", "Avance par dent (fraction du diamètre)", "Avanzamento per dente (frazione del diametro)", "1刃あたりの送り（直径の割合）", "每齿进给（直径的比例）", "Avanço por dente (fração do diâmetro)"],
    ParWorkingMin => ["V-bit and engraver: smallest working diameter (mm)", "Fresa en V y grabador: diámetro de trabajo mínimo (mm)", "V-Fräser und Gravierstichel: kleinster Arbeitsdurchmesser (mm)", "Fraise en V et burin : plus petit diamètre de travail (mm)", "Fresa a V e incisore: diametro di lavoro minimo (mm)", "Vビット・彫刻刀：最小作業径 (mm)", "V 形刀和雕刻刀：最小工作直径 (mm)", "Fresa em V e gravador: menor diâmetro de trabalho (mm)"],
    ParWorkingMax => ["V-bit and engraver: largest working diameter (mm)", "Fresa en V y grabador: diámetro de trabajo máximo (mm)", "V-Fräser und Gravierstichel: größter Arbeitsdurchmesser (mm)", "Fraise en V et burin : plus grand diamètre de travail (mm)", "Fresa a V e incisore: diametro di lavoro massimo (mm)", "Vビット・彫刻刀：最大作業径 (mm)", "V 形刀和雕刻刀：最大工作直径 (mm)", "Fresa em V e gravador: maior diâmetro de trabalho (mm)"],
    ParMinWorking => ["Smallest diameter the chip load is multiplied by (mm)", "Diámetro mínimo por el que se multiplica el avance por diente (mm)", "Kleinster Durchmesser, mit dem der Zahnvorschub multipliziert wird (mm)", "Plus petit diamètre par lequel l'avance par dent est multipliée (mm)", "Diametro minimo per cui si moltiplica l'avanzamento per dente (mm)", "1刃あたりの送りに掛ける最小径 (mm)", "每齿进给所乘的最小直径 (mm)", "Menor diâmetro pelo qual se multiplica o avanço por dente (mm)"],
    ParDrillPlunge => ["Drill plunge factor", "Factor de penetración de la broca", "Eintauchfaktor des Bohrers", "Facteur de plongée du foret", "Fattore di affondamento della punta", "ドリルの突っ込み係数", "钻头下刀系数", "Fator de penetração da broca"],
    ParPlungeRatio => ["Plunge as a fraction of the feed", "Penetración como fracción del avance", "Eintauchen als Anteil des Vorschubs", "Plongée en fraction de l'avance", "Affondamento come frazione dell'avanzamento", "突っ込みを送りの割合で指定", "下刀占进给的比例", "Penetração como fração do avanço"],
    ParPassFraction => ["Depth of a pass (fraction of the diameter)", "Profundidad de pasada (fracción del diámetro)", "Zustelltiefe (Anteil des Durchmessers)", "Profondeur de passe (fraction du diamètre)", "Profondità di passata (frazione del diametro)", "1パスの深さ（直径の割合）", "单次切深（直径的比例）", "Profundidade de passagem (fração do diâmetro)"],
    ParMaxPass => ["Deepest pass (mm)", "Pasada más profunda (mm)", "Tiefste Zustellung (mm)", "Passe la plus profonde (mm)", "Passata più profonda (mm)", "最大の1パス深さ (mm)", "最大单次切深 (mm)", "Passagem mais profunda (mm)"],
    Formulas => ["Formulas and conditions", "Fórmulas y condiciones", "Formeln und Bedingungen", "Formules et conditions", "Formule e condizioni", "計算式と条件", "公式与条件", "Fórmulas e condições"],
    FormPoint => ["A V-bit or an engraver works at the diameter of its point: the diameter {0} mm is kept between {1} and {2} mm, so {3} mm is used", "Una fresa en V o un grabador trabaja con el diámetro de su punta: el diámetro {0} mm se mantiene entre {1} y {2} mm, así que se usa {3} mm", "Ein V-Fräser oder Gravierstichel arbeitet mit dem Durchmesser seiner Spitze: der Durchmesser {0} mm wird auf {1} bis {2} mm begrenzt, verwendet wird {3} mm", "Une fraise en V ou un burin travaille au diamètre de sa pointe : le diamètre {0} mm est maintenu entre {1} et {2} mm, on utilise donc {3} mm", "Una fresa a V o un incisore lavora con il diametro della punta: il diametro {0} mm viene tenuto tra {1} e {2} mm, quindi si usa {3} mm", "Vビットや彫刻刀は先端の径で切削します。径 {0} mm を {1}〜{2} mm の範囲に収め、{3} mm を使います", "V 形刀和雕刻刀按刀尖直径工作：直径 {0} mm 被限制在 {1} 至 {2} mm 之间，因此使用 {3} mm", "Uma fresa em V ou um gravador trabalha com o diâmetro da ponta: o diâmetro {0} mm é mantido entre {1} e {2} mm, por isso usa-se {3} mm"],
    FormRpm => ["speed = surface speed × 1000 / (π × diameter) = {0} × 1000 / (π × {1}) = {2} rpm", "velocidad = velocidad de corte × 1000 / (π × diámetro) = {0} × 1000 / (π × {1}) = {2} rpm", "Drehzahl = Schnittgeschwindigkeit × 1000 / (π × Durchmesser) = {0} × 1000 / (π × {1}) = {2} U/min", "vitesse = vitesse de coupe × 1000 / (π × diamètre) = {0} × 1000 / (π × {1}) = {2} tr/min", "velocità = velocità di taglio × 1000 / (π × diametro) = {0} × 1000 / (π × {1}) = {2} giri/min", "回転数 = 切削速度 × 1000 / (π × 直径) = {0} × 1000 / (π × {1}) = {2} rpm", "转速 = 切削速度 × 1000 / (π × 直径) = {0} × 1000 / (π × {1}) = {2} rpm", "velocidade = velocidade de corte × 1000 / (π × diâmetro) = {0} × 1000 / (π × {1}) = {2} rpm"],
    CondSpindleLimit => ["The spindle turns at most {0} rpm and the material asks for {1} rpm: the speed is lowered to {0} rpm and the feed goes down with it", "El husillo gira como máximo a {0} rpm y el material pide {1} rpm: se baja la velocidad a {0} rpm y el avance baja con ella", "Die Spindel dreht höchstens mit {0} U/min, das Material verlangt {1} U/min: die Drehzahl wird auf {0} U/min gesenkt, und der Vorschub sinkt mit", "La broche tourne à {0} tr/min au plus et la matière demande {1} tr/min : la vitesse est abaissée à {0} tr/min et l'avance baisse avec elle", "Il mandrino gira al massimo a {0} giri/min e il materiale chiede {1} giri/min: la velocità scende a {0} giri/min e l'avanzamento scende con essa", "スピンドルは最大 {0} rpm、材料は {1} rpm を要求します。回転数を {0} rpm に下げ、送りも一緒に下がります", "主轴最高 {0} rpm，而材料需要 {1} rpm：转速降至 {0} rpm，进给随之降低", "O fuso roda no máximo a {0} rpm e o material pede {1} rpm: a velocidade desce para {0} rpm e o avanço desce com ela"],
    CondSpindleFree => ["The spindle can turn that fast ({0} rpm at most): the speed of the material, {1} rpm, is used", "El husillo puede girar tan rápido (máximo {0} rpm): se usa la velocidad del material, {1} rpm", "Die Spindel schafft diese Drehzahl (höchstens {0} U/min): die Drehzahl des Materials, {1} U/min, wird verwendet", "La broche peut tourner aussi vite ({0} tr/min au plus) : on utilise la vitesse de la matière, {1} tr/min", "Il mandrino può girare così veloce ({0} giri/min al massimo): si usa la velocità del materiale, {1} giri/min", "スピンドルはその回転数まで出せます（最大 {0} rpm）。材料の回転数 {1} rpm を使います", "主轴可以达到该转速（最高 {0} rpm）：采用材料的转速 {1} rpm", "O fuso consegue rodar tão depressa (no máximo {0} rpm): usa-se a velocidade do material, {1} rpm"],
    FormFeed => ["feed = speed × flutes × chip fraction × diameter = {0} × {1} × {2} × {3} = {4} mm/min", "avance = velocidad × labios × fracción de viruta × diámetro = {0} × {1} × {2} × {3} = {4} mm/min", "Vorschub = Drehzahl × Schneiden × Spanungsanteil × Durchmesser = {0} × {1} × {2} × {3} = {4} mm/min", "avance = vitesse × dents × fraction de copeau × diamètre = {0} × {1} × {2} × {3} = {4} mm/min", "avanzamento = velocità × taglienti × frazione di truciolo × diametro = {0} × {1} × {2} × {3} = {4} mm/min", "送り = 回転数 × 刃数 × 切りくず比 × 直径 = {0} × {1} × {2} × {3} = {4} mm/min", "进给 = 转速 × 刃数 × 切屑比 × 直径 = {0} × {1} × {2} × {3} = {4} mm/min", "avanço = velocidade × dentes × fração de apara × diâmetro = {0} × {1} × {2} × {3} = {4} mm/min"],
    FormPlungeDrill => ["plunge = speed × {0} × diameter = {1} × {0} × {2} = {3} mm/min", "penetración = velocidad × {0} × diámetro = {1} × {0} × {2} = {3} mm/min", "Eintauchen = Drehzahl × {0} × Durchmesser = {1} × {0} × {2} = {3} mm/min", "plongée = vitesse × {0} × diamètre = {1} × {0} × {2} = {3} mm/min", "affondamento = velocità × {0} × diametro = {1} × {0} × {2} = {3} mm/min", "突っ込み = 回転数 × {0} × 直径 = {1} × {0} × {2} = {3} mm/min", "下刀 = 转速 × {0} × 直径 = {1} × {0} × {2} = {3} mm/min", "penetração = velocidade × {0} × diâmetro = {1} × {0} × {2} = {3} mm/min"],
    FormPlunge => ["plunge = feed × {0} = {1} × {0} = {2} mm/min", "penetración = avance × {0} = {1} × {0} = {2} mm/min", "Eintauchen = Vorschub × {0} = {1} × {0} = {2} mm/min", "plongée = avance × {0} = {1} × {0} = {2} mm/min", "affondamento = avanzamento × {0} = {1} × {0} = {2} mm/min", "突っ込み = 送り × {0} = {1} × {0} = {2} mm/min", "下刀 = 进给 × {0} = {1} × {0} = {2} mm/min", "penetração = avanço × {0} = {1} × {0} = {2} mm/min"],
    FormPass => ["depth of a pass = diameter × {0}, at most {1} mm = {2} mm", "profundidad de pasada = diámetro × {0}, como máximo {1} mm = {2} mm", "Zustelltiefe = Durchmesser × {0}, höchstens {1} mm = {2} mm", "profondeur de passe = diamètre × {0}, au plus {1} mm = {2} mm", "profondità di passata = diametro × {0}, al massimo {1} mm = {2} mm", "1パスの深さ = 直径 × {0}、最大 {1} mm = {2} mm", "单次切深 = 直径 × {0}，最大 {1} mm = {2} mm", "profundidade de passagem = diâmetro × {0}, no máximo {1} mm = {2} mm"],
    CondDrillPass => ["A drill goes through in one pass: there is no depth of a pass", "Una broca atraviesa de una vez: no hay profundidad de pasada", "Ein Bohrer geht in einem Zug durch: es gibt keine Zustelltiefe", "Un foret traverse en une seule passe : pas de profondeur de passe", "Una punta passa da parte a parte in una sola passata: non c'è profondità di passata", "ドリルは1回で貫通するため、1パスの深さはありません", "钻头一次钻透，因此没有单次切深", "Uma broca atravessa de uma só vez: não há profundidade de passagem"],
    KindDrill => ["Drill", "Broca", "Bohrer", "Foret", "Punta", "ドリル", "钻头", "Broca"],
    KindEndMill => ["End mill", "Fresa", "Schaftfräser", "Fraise", "Fresa", "エンドミル", "立铣刀", "Fresa"],
    KindVBit => ["V-bit", "Fresa en V", "V-Fräser", "Fraise en V", "Fresa a V", "Vビット", "V 形刀", "Fresa em V"],
    KindEngraver => ["Engraver", "Grabador", "Gravierstichel", "Burin", "Incisore", "彫刻刀", "雕刻刀", "Gravador"],
    KindLaser => ["Laser", "Láser", "Laser", "Laser", "Laser", "レーザー", "激光", "Laser"],
    KindNozzle => ["Nozzle", "Boquilla", "Düse", "Buse", "Ugello", "ノズル", "喷嘴", "Bico"],
    MatFr4 => ["FR4", "FR4", "FR4", "FR4", "FR4", "FR4", "FR4", "FR4"],
    MatAluminium => ["Aluminium", "Aluminio", "Aluminium", "Aluminium", "Alluminio", "アルミニウム", "铝", "Alumínio"],
    MatAcrylic => ["Acrylic", "Acrílico", "Acryl", "Acrylique", "Acrilico", "アクリル", "亚克力", "Acrílico"],
    MatWood => ["Wood", "Madera", "Holz", "Bois", "Legno", "木材", "木材", "Madeira"],
    MatBrass => ["Brass", "Latón", "Messing", "Laiton", "Ottone", "真鍮", "黄铜", "Latão"],
    Ready => ["Ready", "Listo", "Bereit", "Prêt", "Pronto", "準備完了", "就绪", "Pronto"],
    StatusOpened => ["Opened {0}: {1} tools", "Abierto {0}: {1} herramientas", "{0} geöffnet: {1} Werkzeuge", "{0} ouvert : {1} outils", "Aperto {0}: {1} utensili", "{0} を開きました：工具 {1} 本", "已打开 {0}：{1} 把刀具", "Aberto {0}: {1} ferramentas"],
    StatusSaved => ["Saved {0}: {1} tools", "Guardado {0}: {1} herramientas", "{0} gespeichert: {1} Werkzeuge", "{0} enregistré : {1} outils", "Salvato {0}: {1} utensili", "{0} を保存しました：工具 {1} 本", "已保存 {0}：{1} 把刀具", "Guardado {0}: {1} ferramentas"],
    StatusSavedBackup => ["Saved {0}: {1} tools (the previous file is kept as {0}.bak)", "Guardado {0}: {1} herramientas (el fichero anterior queda como {0}.bak)", "{0} gespeichert: {1} Werkzeuge (die vorherige Datei bleibt als {0}.bak erhalten)", "{0} enregistré : {1} outils (le fichier précédent est conservé sous {0}.bak)", "Salvato {0}: {1} utensili (il file precedente resta come {0}.bak)", "{0} を保存しました：工具 {1} 本（以前のファイルは {0}.bak として残してあります）", "已保存 {0}：{1} 把刀具（原文件保留为 {0}.bak）", "Guardado {0}: {1} ferramentas (o ficheiro anterior fica como {0}.bak)"],
    StatusNewCrib => ["A new empty crib: write the file name and save", "Almacén nuevo y vacío: escriba el nombre del fichero y guarde", "Neuer leerer Bestand: Dateinamen eintragen und speichern", "Nouvel outillage vide : écrivez le nom du fichier et enregistrez", "Nuovo magazzino vuoto: scrivi il nome del file e salva", "空の新しい工具庫です。ファイル名を入力して保存してください", "已新建空刀具库：请输入文件名并保存", "Armazém novo e vazio: escreva o nome do ficheiro e guarde"],
    StatusAdded => ["Tool {1} added: {0}", "Herramienta {1} añadida: {0}", "Werkzeug {1} hinzugefügt: {0}", "Outil {1} ajouté : {0}", "Utensile {1} aggiunto: {0}", "工具 {1} を追加しました：{0}", "已添加刀具 {1}：{0}", "Ferramenta {1} adicionada: {0}"],
    StatusUpdated => ["Tool {1} changed: {0}", "Herramienta {1} modificada: {0}", "Werkzeug {1} geändert: {0}", "Outil {1} modifié : {0}", "Utensile {1} modificato: {0}", "工具 {1} を変更しました：{0}", "已修改刀具 {1}：{0}", "Ferramenta {1} alterada: {0}"],
    StatusRemoved => ["Tool {1} removed: {0}", "Herramienta {1} quitada: {0}", "Werkzeug {1} entfernt: {0}", "Outil {1} retiré : {0}", "Utensile {1} rimosso: {0}", "工具 {1} を削除しました：{0}", "已移除刀具 {1}：{0}", "Ferramenta {1} removida: {0}"],
    StatusUsed => ["Tool {0}: {1}", "Herramienta {0}: {1}", "Werkzeug {0}: {1}", "Outil {0} : {1}", "Utensile {0}: {1}", "工具 {0}：{1}", "刀具 {0}：{1}", "Ferramenta {0}: {1}"],
    StatusImported => ["Imported {0}: {1} tools (not saved yet)", "Importado {0}: {1} herramientas (aún sin guardar)", "{0} importiert: {1} Werkzeuge (noch nicht gespeichert)", "{0} importé : {1} outils (pas encore enregistré)", "Importato {0}: {1} utensili (non ancora salvato)", "{0} を読み込みました：工具 {1} 本（未保存）", "已导入 {0}：{1} 把刀具（尚未保存）", "Importado {0}: {1} ferramentas (ainda por guardar)"],
    StatusExported => ["Exported {0}: {1} tools", "Exportado {0}: {1} herramientas", "{0} exportiert: {1} Werkzeuge", "{0} exporté : {1} outils", "Esportato {0}: {1} utensili", "{0} に書き出しました：工具 {1} 本", "已导出 {0}：{1} 把刀具", "Exportado {0}: {1} ferramentas"],
    StatusFailed => ["Error: {0}", "Error: {0}", "Fehler: {0}", "Erreur : {0}", "Errore: {0}", "エラー：{0}", "错误：{0}", "Erro: {0}"],
    StatusBadSettings => ["The settings file is not valid, the defaults are used: {0}", "El fichero de ajustes no es válido, se usan los valores por defecto: {0}", "Die Einstellungsdatei ist ungültig, die Standardwerte werden verwendet: {0}", "Le fichier de réglages n'est pas valide, les valeurs par défaut sont utilisées : {0}", "Il file delle impostazioni non è valido, si usano i valori predefiniti: {0}", "設定ファイルが無効なため、既定値を使用します：{0}", "设置文件无效，使用默认值：{0}", "O ficheiro de definições não é válido, usam-se os valores predefinidos: {0}"],
    StatusNeedFile => ["Write the name of the file", "Escriba el nombre del fichero", "Geben Sie den Dateinamen ein", "Écrivez le nom du fichier", "Scrivi il nome del file", "ファイル名を入力してください", "请输入文件名", "Escreva o nome do ficheiro"],
    StatusNoSelection => ["Choose a tool first", "Elija primero una herramienta", "Wählen Sie zuerst ein Werkzeug", "Choisissez d'abord un outil", "Scegli prima un utensile", "先に工具を選んでください", "请先选择一把刀具", "Escolha primeiro uma ferramenta"],
    StatusExists => ["{0}: already exists; nothing was overwritten (tick the box to replace it)", "{0}: ya existe; no se ha sobrescrito nada (marque la casilla para reemplazarlo)", "{0}: existiert bereits; nichts wurde überschrieben (Kästchen zum Ersetzen anhaken)", "{0} : existe déjà ; rien n'a été écrasé (cochez la case pour le remplacer)", "{0}: esiste già; non è stato sovrascritto nulla (spunta la casella per sostituirlo)", "{0}：すでに存在します。何も上書きしていません（置き換えるにはチェックを入れてください）", "{0}：已存在，未覆盖任何内容（勾选复选框可替换）", "{0}: já existe; nada foi substituído (marque a caixa para o substituir)"],
}

impl Msg {
    /// The text in a language.
    pub fn tr(self, lang: Lang) -> &'static str {
        self.words()[lang as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sentence_has_a_text_in_each_of_the_eight_languages() {
        for m in Msg::ALL {
            for l in Lang::ALL {
                assert!(!m.tr(l).trim().is_empty(), "{m:?} in {}", l.code());
            }
        }
    }

    #[test]
    fn the_list_of_sentences_has_no_repeats() {
        for (i, a) in Msg::ALL.iter().enumerate() {
            assert!(!Msg::ALL[i + 1..].contains(a), "{a:?} is listed twice");
        }
    }

    #[test]
    fn a_template_has_the_same_places_in_every_language() {
        for m in Msg::ALL {
            let places = |l: Lang| {
                (0..6)
                    .map(|k| m.tr(l).matches(&format!("{{{k}}}")).count())
                    .collect::<Vec<usize>>()
            };
            for l in Lang::ALL {
                assert_eq!(places(l), places(Lang::En), "{m:?} in {}", l.code());
            }
        }
    }

    #[test]
    fn the_places_that_are_filled_are_all_the_places_there_are() {
        for m in Msg::ALL {
            for l in Lang::ALL {
                let text = fill(m.tr(l), &["a", "b", "c", "d", "e", "f"]);
                assert!(
                    !text.contains('{') && !text.contains('}'),
                    "{m:?} in {}",
                    l.code()
                );
            }
        }
    }

    #[test]
    fn the_order_of_the_languages_matches_the_lists() {
        for (k, l) in Lang::ALL.into_iter().enumerate() {
            assert_eq!(l as usize, k);
        }
        assert_eq!(Msg::Save.tr(Lang::Es), "Guardar");
        assert_eq!(Msg::Open.tr(Lang::De), "Öffnen");
        assert_eq!(Msg::Save.tr(Lang::Pt), "Guardar");
        assert_eq!(Msg::TabTools.tr(Lang::En), "Tools");
    }

    #[test]
    fn the_portuguese_is_not_the_brazilian() {
        let pt = |m: Msg| m.tr(Lang::Pt);
        assert!(pt(Msg::CribFile).contains("Ficheiro"));
        assert!(pt(Msg::StatusSaved).contains("Guardado"));
        assert!(!pt(Msg::StatusSaved).contains("Salvo"));
        assert!(!pt(Msg::CribFile).contains("Arquivo"));
    }

    #[test]
    fn a_language_is_found_from_the_code_of_the_system() {
        assert_eq!(Lang::from_code("es_ES.UTF-8"), Some(Lang::Es));
        assert_eq!(Lang::from_code("PT-pt"), Some(Lang::Pt));
        assert_eq!(Lang::from_code("ja"), Some(Lang::Ja));
        assert_eq!(Lang::from_code("xx"), None);
        assert_eq!(Lang::from_code(""), None);
    }

    #[test]
    fn the_places_of_a_template_are_filled() {
        assert_eq!(fill("{1} and {0}", &["a", "b"]), "b and a");
        assert_eq!(fill("none", &["a"]), "none");
    }
}
