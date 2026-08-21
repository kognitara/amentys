#![cfg_attr(not(test), no_std)]

#[derive(Clone, Debug, Copy)]
pub struct Arg {
    pub name: &'static str,
    pub short: Option<&'static str>,
    pub help: &'static str,
    pub long: Option<&'static str>,
    pub required: bool,
}

impl Arg {
    /// Initialise un nouvel argument
    pub const fn new(name: &'static str, help: &'static str) -> Self {
        Self {
            name,
            short: None,
            long: None,
            required: false,
            help,
        }
    }
    pub const fn short(mut self, s: &'static str) -> Self {
        self.short = Some(s);
        self
    }

    pub const fn long(mut self, l: &'static str) -> Self {
        self.long = Some(l);
        self
    }

    pub fn required(mut self, arg: bool) -> Self {
        self.required = arg;
        self
    }
}

// =========================================================
// 2. DÉFINITION D'UNE COMMANDE (L'équivalent de clap::Command)
// =========================================================

#[derive(Clone, Debug, Copy)]
pub struct Subcommand {
    pub name: &'static str,
    pub about: &'static str,
    pub args: [Option<Arg>; 8],
    pub arg_count: usize,
}

impl Subcommand {
    pub const fn new(name: &'static str, about: &'static str) -> Self {
        Self {
            name,
            about,
            args: [None; 8],
            arg_count: 0,
        }
    }

    pub fn arg(mut self, arg: Arg) -> Self {
        if self.arg_count < self.args.len() {
            self.args[self.arg_count] = Some(arg);
            self.arg_count += 1;
        }
        self
    }
}

#[derive(Clone, Debug, Copy)]
pub struct Command {
    pub name: &'static str,
    pub about: &'static str,
    pub args: [Option<Arg>; 8],
    arg_count: usize,
    sub_count: usize,
    pub subcommands: [Option<Subcommand>; 8],
}

impl Command {
    pub const fn new(name: &'static str, about: &'static str) -> Self {
        Self {
            name,
            about,
            args: [None; 8],
            arg_count: 0,
            sub_count: 0,
            subcommands: [None; 8],
        }
    }

    pub fn arg(mut self, arg: Arg) -> Self {
        if self.arg_count < self.args.len() {
            self.args[self.arg_count] = Some(arg);
            self.arg_count += 1;
        }
        self
    }

    pub fn subcommand(mut self, sub: Subcommand) -> Self {
        if self.sub_count < self.subcommands.len() {
            self.subcommands[self.sub_count] = Some(sub);
            self.sub_count += 1;
        }
        self
    }
    
    pub fn parse(&self, x: &str) -> Matches<'_> {
        todo!()
    }
}

// =========================================================
// 3. RÉSULTAT DU PARSING (L'équivalent de clap::ArgMatches)
// =========================================================

pub struct Matches<'a> {
    pub subcommand_name: Option<&'a str>,
    // On stocke les paires (nom_argument, valeur_trouvée)
    pub values: [Option<(&'a str, &'a str)>; 8],
    value_count: usize,
}

impl<'a> Matches<'a> {
    /// Récupère la valeur d'un argument via son nom
    pub fn get_one(&self, arg_name: &str) -> Option<&'a str> {
        for i in 0..self.value_count {
            if let Some((name, val)) = self.values[i]
                && name == arg_name
            {
                return Some(val);
            }
        }
        None
    }
}

// =========================================================
// 4. LA LOGIQUE DE PARSING
// =========================================================

impl Command {
    /// Analyse la liste brute fournie par Linux ou Amentys
    pub fn get_matches(&self, raw_args: &[&'static str]) -> Matches<'static> {
        let mut matches = Matches {
            subcommand_name: None,
            values: [None; 8],
            value_count: 0,
        };

        if raw_args.len() <= 1 {
            return matches;
        }

        // 1. Est-ce qu'on appelle une sous-commande ? (ex: "we commit")
        let first_arg = raw_args[1];
        let mut active_args = &self.args;

        for i in 0..self.sub_count {
            if let Some(sub) = &self.subcommands[i]
                && sub.name == first_arg
            {
                matches.subcommand_name = Some(sub.name);
                active_args = &sub.args; // On passe aux arguments de la sous-commande
                break;
            }
        }

        // 2. Parcourir le reste pour extraire les valeurs (ex: "-m" "Mon texte")
        let mut i = if matches.subcommand_name.is_some() {
            2
        } else {
            1
        };

        while i < raw_args.len() {
            let current_raw = raw_args[i];

            for arg_def in active_args.iter().flatten() {
                if (Some(current_raw) == arg_def.short || Some(current_raw) == arg_def.long)
                    && arg_def.required
                    && i + 1 < raw_args.len()
                {
                    // On a trouvé notre valeur !
                    matches.values[matches.value_count] = Some((arg_def.name, raw_args[i + 1]));
                    matches.value_count += 1;
                    i += 1; // On saute la valeur pour le prochain tour de boucle
                }
            }
            i += 1;
        }
        matches
    }
}
