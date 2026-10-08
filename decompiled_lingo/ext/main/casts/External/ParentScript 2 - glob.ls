property managers, nodeData, soundTimer, closeRequested, escPressed, initialized
global glob

on new me
  glob = me
  initSound()
  managers = [:]
  managers[#config] = script("config_manager").new(me)
  managers[#time] = script("time_manager").new(me)
  managers[#sprite] = script("sprite2d_manager").new(me)
  managers[#node] = script("node_manager").new(me)
  managers.node.setSpriteManager(managers.sprite)
  managers[#mouse] = script("mouse_manager").new(me)
  managers[#game] = script("game_manager").new(me)
  managers[#mode] = script("mode_manager").new(me)
  managers[#savegame] = script("savegame_manager").new(me)
  managers[#dnd] = script("dnd_manager").new(me)
  managers[#view] = script("view_manager").new(me)
  managers[#event] = script("event_manager").new(me)
  managers[#config].init()
  managers[#savegame].init()
  nodeData = [:]
  return me
end

on init me
  updateStage()
  if managers.savegame.expired then
    return 
  end if
  initialized = 1
  me.loadNodeData()
  m = the milliSeconds
  repeat with i in Keys(managers)
    m1 = the milliSeconds
    managers[i].init()
    go(the frame)
    put "init " & i & " manager took " & the milliSeconds - m1
  end repeat
  put "init managers took " & the milliSeconds - m
  n = managers.node
  g = n.getGroup("game")
  t = n.getGroup("titles")
  t.moveToFront()
  managers.time.goFrame("play")
  managers.mode.setMode(#menu)
  soundTimer = script("timeout").new(250, [#target: script("SoundCode", "sfx"), #handler: #SndCheckPlaylist], 0, "sound")
end

on destroy me
  if not initialized then
    return 
  end if
  if not voidp(soundTimer) then
    soundTimer.destroy()
    soundTimer = VOID
  end if
  call(#destroy, managers)
  managers.deleteAll()
end

on loadNodeData me
  txt = member("layout_list", "scripts.layout")
  repeat with i = 1 to txt.line.count
    fr = txt.line[i]
    nodeData[fr] = me.getNodeData(fr)
  end repeat
end

on getNodeData me, nm
  if voidp(nodeData) then
    return 
  end if
  r = nodeData[nm]
  if not voidp(r) then
    return r
  end if
  m = member(nm & "_layout", "scripts.layout")
  if m.number = -1 then
    return 
  end if
  r = value(devoid(m.text))
  me.fillDummies(r)
  nodeData[nm] = r
  return r
end

on fillDummies me, nd
  if voidp(nd) then
    return 
  end if
  if not voidp(nd[#dummyID]) and (nd[#dummyID] <> EMPTY) then
    d = me.getNodeData(nd.dummyID)
    if not voidp(d) then
      nd.children = d.children
    else
      nothing()
    end if
    nd[#dummyID] = VOID
    nd[#member] = VOID
  end if
  if voidp(nd[#children]) then
    return 
  end if
  repeat with i in nd.children
    me.fillDummies(i)
  end repeat
end

on start me
  go(marker("load"))
  me.init()
end

on mouseDown me
  managers.mouse.mouseDown()
end

on mouseUp me
  managers.mouse.mouseUp()
end

on keyDown me, k
  managers.event.sendEvent(#keyDown, k)
  escPressed = the keyCode = 53
end

on keyUp me
end

on exitFrame me
  managers.time.exitFrame()
  if closeRequested and not escPressed then
    halt()
  end if
  closeRequested = 0
  escPressed = 0
end

on setMode me, m, data
  managers.mode.setMode(m)
end
