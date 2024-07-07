CREATE MIGRATION m1d7oldgugzthx5qtldfmokhrxhsjmxwrbynfetsgq7fibzrf3n66q
    ONTO initial
{
  CREATE MODULE fate IF NOT EXISTS;
  CREATE TYPE default::Todos {
      CREATE REQUIRED PROPERTY todo: std::str;
  };
  CREATE TYPE fate::AllowedSkill {
      CREATE REQUIRED PROPERTY name: std::str {
          CREATE CONSTRAINT std::exclusive;
          CREATE CONSTRAINT std::regexp("^[A-Z0-9 ']+");
      };
  };
  CREATE TYPE fate::Skill {
      CREATE REQUIRED LINK name: fate::AllowedSkill {
          SET readonly := true;
      };
      CREATE REQUIRED PROPERTY level: std::int16 {
          SET default := -1;
      };
  };
  CREATE SCALAR TYPE fate::AspectType EXTENDING enum<High, Trouble, Other>;
  CREATE TYPE fate::Aspect {
      CREATE REQUIRED PROPERTY aspect_type: fate::AspectType;
      CREATE REQUIRED PROPERTY description: std::str;
  };
  CREATE TYPE fate::PC {
      CREATE MULTI LINK aspects: fate::Aspect;
      CREATE MULTI LINK skills: fate::Skill;
      CREATE REQUIRED PROPERTY name: std::str;
      CREATE MULTI PROPERTY stunts: std::str;
  };
};
