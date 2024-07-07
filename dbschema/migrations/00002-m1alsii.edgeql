CREATE MIGRATION m1alsiinwq2ftid2rk5xqexbgy2z7yugxrx2nftncr6d5gcu73pxba
    ONTO m1d7oldgugzthx5qtldfmokhrxhsjmxwrbynfetsgq7fibzrf3n66q
{
  ALTER TYPE fate::AllowedSkill {
      DROP PROPERTY name;
  };
  ALTER TYPE fate::Skill {
      DROP LINK name;
      DROP PROPERTY level;
  };
  DROP TYPE fate::AllowedSkill;
  ALTER TYPE fate::Aspect {
      DROP PROPERTY aspect_type;
      DROP PROPERTY description;
  };
  DROP TYPE fate::PC;
  DROP TYPE fate::Aspect;
  DROP TYPE fate::Skill;
  DROP SCALAR TYPE fate::AspectType;
};
