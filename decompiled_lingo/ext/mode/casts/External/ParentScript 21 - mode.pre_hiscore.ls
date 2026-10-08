property mgr, name, screen, node, grp, gameNode, buttons, initialized, submittedRank, comicNode

on new me, m
  mgr = m
  grp = getNodeGroup("titles")
  name = #pre_hiscore
  return me
end

on init me
  if initialized then
    return 
  end if
  initialized = 1
  node = grp.newNode(getNodeData("hiscore_popup"))
  node.setVisible(0)
  node.setZOffset(1000000)
  buttons = [:]
  repeat with i in [#hiscore, #menu]
    buttons[i] = newButton([#node: node[string(i)], #callback: [#target: me, #handler: #goMode, #Args: i], #members: [#rollover: member("hsp_" & i & "_on", "gfx.screens")]])
  end repeat
end

on clickButton me
end

on destroy me
  if initialized then
    call(#destroy, buttons)
    node.destroy()
  end if
  voidRefs(me)
end

on goMode me, m
  if m = #hiscore then
    setMode(#hiscore, submittedRank)
  else
    if m = #menu then
      setMode(#menu)
    end if
  end if
end

on enterMode me, previousMode, sr
  if previousMode.name = #end_comic then
    comicNode = previousMode.node
  else
    comicNode = VOID
  end if
  submittedRank = sr
  if not initialized then
    me.init()
  else
    node.embody()
  end if
  node.updateLocZ()
  grp.moveToFront()
  node.moveToFront()
  grp.setVisible(1)
  node.setVisible(1)
end

on event me, sym
end

on leaveMode me, nextMode
  if not voidp(comicNode) then
    comicNode.setVisible(0)
    comicNode.disembody()
  end if
  getNodeGroup("game").setVisible(0)
  node.setVisible(0)
  node.disembody()
end
