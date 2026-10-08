property mgr, name, save_mgr, node_mgr, node, grp, screen, buttons, okbutton, home, offscreen, lastMode, initialized
global glob

on new me, m
  mgr = m
  node_mgr = m.glob.managers.node
  offscreen = point(-5000, -5000)
  name = #options
  return me
end

on init me
  if initialized then
    return 
  end if
  initialized = 1
  grp = node_mgr.getGroup("titles")
  node = grp.newNode(getNodeData("options"))
  home = node.getLoc()
  node.setVisible(0)
  node.setLoc(offscreen)
  node.setZOffset(999999)
  save_mgr = glob.managers.savegame
  buttons = [:]
  repeat with i in ["music", "sfx", "fs"]
    bls = [:]
    repeat with j in ["off", "on"]
      bn = node[i & "_" & j]
      dat = [i, j]
      hmem = member("opt_" & j & "_hilite", "gfx.screens")
      b = newButton([#node: bn, #members: [#normal: member("opt_" & j, "gfx.screens"), #rollover: hmem, #disabled: hmem], #callback: [#target: me, #handler: symbol("set" & i), #Args: j = "on"]])
      bls[j] = b
    end repeat
    buttons[i] = bls
  end repeat
  Ok = node["okbutton"]
  okbutton = newButton([#node: Ok, #members: [#normal: member("opt_ok", "gfx.screens"), #rollover: member("opt_ok_rollover", "gfx.screens")], #callback: [#target: me, #handler: #hide]])
end

on clickButton me, Args
end

on setMusic me, b
  buttons["music"]["on"].setDisabled(b)
  buttons["music"]["off"].setDisabled(not b)
  save_mgr.options.music = b
  save_mgr.updateOptions()
  save_mgr.saveState()
end

on setSFX me, b
  buttons["sfx"]["on"].setDisabled(b)
  buttons["sfx"]["off"].setDisabled(not b)
  save_mgr.options.sfx = b
  save_mgr.updateOptions()
  save_mgr.saveState()
end

on setFS me, b
  buttons["fs"]["on"].setDisabled(b)
  buttons["fs"]["off"].setDisabled(not b)
  save_mgr.options.fullscreen = b
  save_mgr.updateOptions()
  save_mgr.saveState()
end

on hide me
  mgr.setMode(lastMode)
end

on destroy me
  if initialized then
    repeat with i in buttons
      call(#destroy, i)
      i.deleteAll()
    end repeat
    okbutton.destroy()
    node.destroy()
  end if
  voidRefs(me)
end

on enterMode me, previousMode
  if not initialized then
    me.init()
  else
    node.embody()
  end if
  node.updateLocZ()
  lastMode = previousMode.name
  opt = save_mgr.options
  buttons["music"]["on"].setDisabled(opt.music)
  buttons["music"]["off"].setDisabled(not opt.music)
  buttons["sfx"]["on"].setDisabled(opt.sfx)
  buttons["sfx"]["off"].setDisabled(not opt.sfx)
  buttons["fs"]["on"].setDisabled(opt.fullscreen)
  buttons["fs"]["off"].setDisabled(not opt.fullscreen)
  grp.setVisible(1)
  node.setVisible(1)
  node.setLoc(home)
end

on event me, sym
end

on leaveMode me, nextMode
  node.disembody()
  node.setVisible(0)
  node.setLoc(offscreen)
  save_mgr.updateOptions()
end
