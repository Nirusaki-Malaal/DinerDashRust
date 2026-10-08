property mgr, name, node_mgr, game_mgr, screen, node, grp, page, initialized, arrow

on new me, m
  mgr = m
  node_mgr = m.glob.managers.node
  game_mgr = m.glob.managers.game
  grp = node_mgr.getGroup("titles")
  name = #transform_comic
  return me
end

on init me
  if initialized then
    return 
  end if
  initialized = 1
  node = grp.newNode(getNodeData("transform_comic"))
  arrow = newButton([#node: node["arrow"], #members: [#rollover: member("help_arrow")], #callback: [#target: me, #handler: #mousePress]])
  node.setVisible(0)
end

on destroy me
  if initialized then
    arrow.destroy()
    node.destroy()
  end if
  voidRefs(me)
end

on enterMode me, previousMode
  game_mgr.stopGame()
  getNodeGroup("game").setVisible(0)
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
  if page = 2 then
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
end
