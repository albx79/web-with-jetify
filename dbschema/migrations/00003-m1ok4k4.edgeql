CREATE MIGRATION m1ok4k4gwwckcvsfh5zjw3uryyvbl6ztk2yl6z3fj6imftiw4nfyja
    ONTO m1alsiinwq2ftid2rk5xqexbgy2z7yugxrx2nftncr6d5gcu73pxba
{
  CREATE SCALAR TYPE fate::AspectType EXTENDING enum<High, Trouble, Other>;
  CREATE SCALAR TYPE fate::Skill EXTENDING std::str {
      CREATE CONSTRAINT std::regexp("^[A-Z0-9 ']+");
  };
  CREATE TYPE fate::PC {
      CREATE REQUIRED PROPERTY aspects: array<tuple<fate::AspectType, std::str>>;
      CREATE REQUIRED PROPERTY skills: array<tuple<fate::Skill, std::int16>>;
      CREATE REQUIRED PROPERTY name: std::str;
      CREATE REQUIRED PROPERTY stunts: array<std::str>;
  };
  CREATE TYPE fate::Aspect {
      CREATE REQUIRED PROPERTY aspect_type: fate::AspectType;
      CREATE REQUIRED PROPERTY description: std::str;
  };
  CREATE TYPE fate::Game {
      CREATE MULTI LINK pcs: fate::PC;
      CREATE REQUIRED PROPERTY allowed_skills: array<std::str>;
      CREATE REQUIRED PROPERTY title: std::str;
  };
};
