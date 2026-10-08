property color, numbers, multiplier, members, node, showing, home_yreg, speed, height, y, alpha, view_mgr, type
global glob

on new me
  node = glob.managers.game.node.newNode(getNodeData("colorbonus"))
  n = node
  view_mgr = glob.managers.view
  type = "colorbonus"
  color = n["color"]
  numbers = []
  repeat with i = 1 to 4
    numbers.add(n["number_" & i])
  end repeat
  multiplier = n["multiplier"]
  members = [:]
  colors = [#red, #green, #purple, #blue, #yellow]
  members = [:]
  repeat with i in colors
    ls = [:]
    ls[#color] = member(i & ".color", "gfx.numbers")
    nums = []
    repeat with num = 0 to 9
      nums.add(member(i & "." & num, "gfx.numbers"))
    end repeat
    ls[#numbers] = nums
    mult = []
    repeat with num = 1 to 5
      mult.add(member(i & ".x" & num, "gfx.numbers"))
    end repeat
    ls[#multiplier] = mult
    members[i] = ls
  end repeat
  node.setZOffset(1000000)
  node.setRegPoint(numbers[3].getLoc() + [0, 10])
  home_yreg = node.getYReg()
  speed = 100.0
  height = 75.0
  node.setVisible(0)
  return me
end

on show me, Args
  num = Args.score
  mult = Args.multiplier
  col = Args.color
  if not num.integerp then
    return 
  end if
  if (num > 9999) or (num < 0) then
    return 
  end if
  mems = members[col]
  if voidp(mems) then
    return 
  end if
  mult_mems = mems.multiplier
  num_mems = mems.numbers
  str = string(num)
  ct = str.char.count
  diff = 4 - ct
  repeat with i = 1 to 4
    n = numbers[i]
    if i <= diff then
      n.setVisible(0)
      next repeat
    end if
    n.setMember(num_mems[integer(str.char[i - diff]) + 1])
    n.setVisible(1)
  end repeat
  mult = max(1, min(mult, 5))
  multiplier.setMember(mult_mems[mult])
  color.setMember(mems.color)
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
  node = VOID
  unsubscribe(me, #exitFrame)
end
