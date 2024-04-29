module default {
    type Todos {
        required todo: str
    }
}

module fate {
    type AllowedSkill {
        required name: str {
            constraint exclusive;
            constraint regexp(r"^[A-Z0-9 ']+");
        }
    }

    type Skill {
        required name: AllowedSkill {
            readonly := true;
        };
        required level: int16 {
            default := -1;
        };
    }

    scalar type AspectType extending enum<High, Trouble, Other>;

    type Aspect {
        required aspect_type: AspectType;
        required description: str;
    }

    type PC {
        required name: str;
        multi skills: Skill;
        multi aspects: Aspect;
        multi stunts: str;
    }
}
