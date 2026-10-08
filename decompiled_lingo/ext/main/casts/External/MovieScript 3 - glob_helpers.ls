global glob

on setMode m, data
  if voidp(glob) then
    return 
  end if
  glob.managers.mode.setMode(m, data)
end

on getDragGroup str
  if voidp(glob) then
    return 
  end if
  return glob.managers.dnd.getGroup(str)
end

on getGroupConfig str
  if voidp(glob) then
    return 
  end if
  c = glob.managers.config.group.types[str]
  if voidp(c) then
    complain("no group type " & str)
  end if
  return c
end

on getNodeGroup str
  if voidp(glob) then
    return 
  end if
  return glob.managers.node.getGroup(str)
end

on sendEvent sym, Args
  if voidp(glob) then
    return 
  end if
  glob.managers.event.sendEvent(sym, Args)
end

on subscribe obj, evt, grp
  if voidp(glob) then
    return 
  end if
  glob.managers.event.subscribe(obj, evt, grp)
end

on unsubscribe obj, evt, grp
  if voidp(glob) then
    return 
  end if
  glob.managers.event.unsubscribe(obj, evt, grp)
end

on setcursor c
  if voidp(glob) then
    return 
  end if
  glob.managers.mouse.setcursor(c)
end

on winLevel me
  glob.managers.game.winLevel()
end

on loseLevel me
  glob.managers.game.loseLevel()
end

on loseGame me
  glob.managers.game.loseGame()
end

on endsub me
  glob.managers.game.endSubLevel()
end

on refreshConfigs me
  put "refreshing configs..."
  repeat with i in ["global", "career", "endless", "powerups", "customer"]
    repeat with j in ["_date", "_dump"]
      member(i & j, "scripts.config").text = EMPTY
    end repeat
  end repeat
end

on newButton Args
  return glob.managers.node.newButton(Args)
end

on newMouseListener Args
  return glob.managers.node.newMouseListener(Args)
end

on setalert bool
  if bool then
    the alertHook = script("show_alerts")
  else
    the alertHook = script("hide_alerts")
  end if
end
