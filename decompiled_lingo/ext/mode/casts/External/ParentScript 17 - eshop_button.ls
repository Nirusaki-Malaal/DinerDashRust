property icon, name, desc, members, type, desc_string, mgr, mouselistener, mouseinside, Info

on new me, Args
  icon = Args.icon
  mouselistener = newMouseListener([#node: icon])
  name = Args.name
  desc = Args.desc
  mgr = Args.mgr
  members = [:]
  repeat with i in [#podium, #stereo, #shoes, #oven, #maitred, #drinks, #table]
    members[i] = [#normal: member("endshop_" & i), #rollover: member("endshop_" & i & "_on")]
  end repeat
  return me
end

on destroy me
  mouselistener.destroy()
  repeat with i = 1 to me.count
    me[#i] = VOID
  end repeat
end

on setDisabled me
  mouselistener.clearCallbacks()
end

on setItem me, inf
  if voidp(inf) then
    name.setVisible(0)
    icon.setVisible(0)
    mouseinside = 0
    mouselistener.clearCallbacks()
    type = #none
    inf = VOID
    return 
  end if
  type = inf.type
  Info = inf
  name.setText(string(inf[#name]))
  name.setVisible(1)
  desc_string = string(inf[#description])
  icon.setMember(members[type].normal)
  icon.setColor(rgb(255, 255, 255))
  mouselistener.forwardEvents(me)
  icon.setVisible(1)
  if icon.mouseWithin() then
    me.mouseLeave()
    me.mouseEnter()
  end if
end

on mouseEnter me
  mouseinside = 1
  desc.setText(desc_string)
  desc.setVisible(1)
  icon.setColor(rgb(255, 255, 0))
  setcursor(#finger_up)
end

on mouseLeave me
  mouseinside = 0
  icon.setColor(rgb(255, 255, 255))
  setcursor(#arrow)
  desc.setText(EMPTY)
end

on mouseDown me
  if not mouseinside then
    return 
  end if
  mgr.selectItem(Info, icon)
  setcursor(#finger_down)
end

on mouseUp me
  setcursor(#finger_up)
end
