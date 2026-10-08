property mgr, name, node_mgr, screen, node, grp, menubutton, initialized, page, buttons

on new me, m
  mgr = m
  node_mgr = m.glob.managers.node
  grp = node_mgr.getGroup("titles")
  name = #help
  return me
end

on init me
  if initialized then
    return 
  end if
  initialized = 1
  node = grp.newNode(getNodeData("help"))
  node.setVisible(0)
  menubutton = newButton([#node: node["menubutton"], #callback: [#target: mgr, #handler: #setMode, #Args: #menu], #members: [#rollover: member("help_button_backtomenu_up", "gfx.screens")]])
  arrowmems = [#normal: member("help_arrow"), #rollover: member("help_arrow_up")]
  buttons = [:]
  buttons[#previous] = newButton([#node: node["previous"], #members: arrowmems, #callback: [#target: me, #handler: #goPrevious]])
  buttons[#next] = newButton([#node: node["next"], #members: arrowmems, #callback: [#target: me, #handler: #goNext]])
  buttons[#credits] = newButton([#node: node["credits"], #callback: [#target: me, #handler: #goCredits], #members: [#rollover: member("help_button_credits_up", "gfx.screens")]])
  buttons[#instructions] = newButton([#node: node["instructions"], #callback: [#target: me, #handler: #goInstructions], #members: [#rollover: member("help_button_instruction_up", "gfx.screens")]])
end

on destroy me
  if initialized then
    menubutton.destroy()
    node.destroy()
  end if
  voidRefs(me)
end

on goPrevious me
  page = max(1, page - 1)
  node.setAnimation(string(page))
  buttons.previous.setVisible(page > 1)
  buttons.next.setVisible(page < 5)
end

on goNext me
  page = min(5, page + 1)
  node.setAnimation(string(page))
  buttons.previous.setVisible(page > 1)
  buttons.next.setVisible(page < 5)
end

on goCredits me
  node.setAnimation("credits")
  buttons.previous.setVisible(0)
  buttons.next.setVisible(0)
  buttons.credits.setVisible(0)
  buttons.instructions.setVisible(1)
end

on goInstructions me
  page = 1
  node.setAnimation("1")
  buttons.previous.setVisible(0)
  buttons.next.setVisible(1)
  buttons.credits.setVisible(1)
  buttons.instructions.setVisible(0)
end

on enterMode me, previousMode
  if not initialized then
    me.init()
  else
    node.embody()
  end if
  node.updateLocZ()
  grp.setVisible(1)
  node.setVisible(1)
  buttons.next.setCallback([#target: me, #handler: #goNext])
  buttons.previous.setCallback([#target: me, #handler: #goPrevious])
  me.goInstructions()
end

on event me, sym
end

on leaveMode me, nextMode
  node.disembody()
  if nextMode.name <> #menu then
    grp.setVisible(0)
  end if
  node.setVisible(0)
end
