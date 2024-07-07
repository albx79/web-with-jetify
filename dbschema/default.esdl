module default {
    type Todos {
        required todo: str
    }
}

module fate {

    scalar type Skill extending str {
        constraint regexp(r"^[A-Z0-9 ']+");
    }

    scalar type AspectType extending enum<High, Trouble, Other>;

    type Aspect {
        required aspect_type: AspectType;
        required description: str;
    }

    type PC {
        required name: str;
        required skills: array<tuple<Skill, int16>>;
        required aspects: array<tuple<AspectType, str>>;
        required stunts: array<str>;
    }

    type Game {
        required title: str;
        required allowed_skills: array<str>;
        multi link pcs: PC;
    }
}
