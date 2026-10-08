property numbers, members, node, showing, home_yreg, speed, height, y, alpha, view_mgr, type
global glob

on new me
  node = glob.managers.game.node.newNode(getNodeData("penalty"))
  n = node
  view_mgr = glob.managers.view
  type = "penalty"
  numbers = []
  repeat with i = 1 to 4
    numbers.add(node["number_" & i])
  end repeat
  members = []
  repeat with i = 0 to 9
    members.add(member("negative." & i, "gfx.numbers"))
  end repeat
  node.setZOffset(1000000)
  node.setXReg(node["number_3"].getLoc()[1])
  home_yreg = node.getYReg()
  speed = 50.0
  height = 50.0
  node.setVisible(0)
  return me
end

on show me, num
  if (num > 9999) or (num < 0) then
    return 
  end if
  str = string(num)
  ct = str.char.count
  repeat with i = 1 to 4
    n = numbers[i]
    if i > ct then
      n.setVisible(0)
      next repeat
    end if
    n.setMember(members[integer(str.char[i]) + 1])
    n.setVisible(1)
  end repeat
  y = 0
  alpha = 100
  node.setYReg(home_yreg)
  node.setVisible(1)
  subscribe(me, #exitFrame, "score")
end

on hide me
  node.setVisible(0)
  unsubscribe(me, #exitFrame, "score")
  view_mgr.returnView(me)
end

on setObject me
end

on removeObject me
end

on exitFrame me, ms
  y = y + (ms * 0.001 * speed)
  if y > height then
    me.hide()
    return 
  end if
  node.setYReg(home_yreg + y)
end

on destroy me
  numbers.deleteAll()
  bonus = VOID
  color = VOID
  multiplier = VOID
  node.destroy()
  repeat with i = 1 to me.count
    me[i] = VOID
  end repeat
  unsubscribe(me, #exitFrame, "score")
end
