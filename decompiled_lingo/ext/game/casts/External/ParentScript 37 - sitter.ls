property color, chair, chain, chained, sitterGroup, ready, mood, views, state, game_mgr, chairBonus

on new me, Args
  views = []
  sitterGroup = Args.sitterGroup
  game_mgr = sitterGroup.game_mgr
  chairBonus = getConfigData().baseChairBonus
  cfg = Args.config
  repeat with i = 1 to Args.config.count
    me.setProp(cfg.getPropAt(i), cfg[i])
  end repeat
  color = Args.color
  if sitterGroup.graphicType = "critic" then
    nothing()
  end if
  ready = 0
  mood = 3
  return me
end

on setState me, sym
  state = sym
  call(#setState, views, state)
end

on setReady me
  call(#update, views)
end

on drink me
  call(#drink, views)
end

on setFoodPickedUp me, b
  call(#setFoodPickedUp, views, b)
end

on addView me, vu
  if views.getOne(vu) = 0 then
    views.add(vu)
  end if
end

on removeView me, vu
  views.deleteOne(vu)
end

on destroy me
  call(#setSitter, views, VOID)
  game_mgr = VOID
  views.deleteAll()
  chain = VOID
  sitterGroup = VOID
end

on checkChairBonus me
  if chair = VOID then
    return 
  end if
  chair.checkBonus()
end

on setChair me, ch
  chair = ch
end

on update me
  ready = sitterGroup.state.name = #ready
  mood = sitterGroup.mood
  call(#update, views)
end
