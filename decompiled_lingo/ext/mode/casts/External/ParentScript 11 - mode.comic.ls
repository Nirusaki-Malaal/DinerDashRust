property mgr, name, node_mgr, game_mgr, screen, node, grp, page, initialized, arrow

on new me, m
  mgr = m
  node_mgr = m.glob.managers.node
  game_mgr = m.glob.managers.game
  grp = node_mgr.getGroup("titles")
  name = #comic
  return me
end

on init me
  if initialized then
    return 
  end if
  initialized = 1
  node = grp.newNode(getNodeData("comic"))
  node.setVisible(0)
  arrow = newButton([#node: node["arrow"], #members: [#rollover: member("help_arrow")], #callback: [#target: me, #handler: #mousePress]])
end

on destroy me
  if initialized then
    node.destroy()
    arrow.destroy()
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
  page = 1
  grp.setVisible(1)
  node.setAnimation("page_" & page)
  node.setVisible(1)
end

on mousePress me
  if page = 3 then
    game_mgr.setPlayMode(#career)
    game_mgr.newGame([#stage: 1, #level: 1])
    me.leaveMode()
    game_mgr.loadLevel()
    mgr.setMode(#startLevel)
    return 
  end if
  page = page + 1
  node.setAnimation("page_" & page)
end

on event me, sym
end

on leaveMode me, nextMode
  node.disembody()
  node.setVisible(0)
  unsubscribe(me, #mousePress, "titles")
end
