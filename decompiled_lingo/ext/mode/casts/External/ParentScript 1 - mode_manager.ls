property glob, currentMode, modes, game_mgr

on new me, g
  glob = g
  game_mgr = glob.managers.game
  modes = [:]
  return me
end

on init me
  repeat with i in [#comic, #menu, #transform_comic, #end_comic, #endless, #login, #endless_shop, #career, #hiscore, #startstage, #startLevel, #play, #endlevel, #quit, #pause, #pre_hiscore, #help, #options]
    modes[i] = script("mode." & string(i), "scripts.mode").new(me)
  end repeat
end

on destroy me
  call(#destroy, modes)
  modes.deleteAll()
  glob = VOID
end

on setMode me, sym, data, revert
  m = modes[sym]
  if voidp(m) then
    complain("no mode called " & sym)
    return 
  end if
  if currentMode = m then
    return 
  end if
  if not voidp(currentMode) and (sym <> #quit) and (sym <> #pause) then
    currentMode.leaveMode(m)
  end if
  if not revert then
    m.enterMode(currentMode, data)
  end if
  currentMode = m
end
