property chain, numbers, multiplier, plus, num_mems, mult_mems, type_mems, node, showing, home_yreg, speed, height, y, alpha, view_mgr, type
global glob

on new me
  node = glob.managers.game.node.newNode(getNodeData("multbonus"))
  n = node
  view_mgr = glob.managers.view
  type = "multbonus"
  chain = n["chain"]
  plus = n["plus"]
  numbers = []
  repeat with i = 1 to 4
    numbers.add(n["number_" & i])
  end repeat
  multiplier = n["multiplier"]
  members = [:]
  num_mems = []
  repeat with num = 0 to 9
    num_mems.add(member("tip." & num, "gfx.numbers"))
  end repeat
  mult_mems = []
  repeat with num = 1 to 12
    mult_mems.add(member("chain.x" & num, "gfx.numbers"))
  end repeat
  type_mems = [:]
  repeat with i in [#food, #ticket, #order, #dishes, #check]
    type_mems[i] = member("chain." & i)
  end repeat
  node.setZOffset(1000000)
  node.setXReg(-15)
  home_yreg = node.getYReg()
  speed = 150.0
  height = 90.0
  node.setVisible(0)
  return me
end

on show me, Args
  num = Args.score
  mult = Args[#multiplier]
  tp = Args[#type]
  if not num.integerp then
    return 
  end if
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
    n.setMember(num_mems[integer(str.char[i]) + 1])
    n.setVisible(1)
  end repeat
  if mult = 0 then
    multiplier.setVisible(0)
    chain.setVisible(0)
  else
    mult = max(1, min(mult, 12))
    multiplier.setMember(mult_mems[mult])
    multiplier.setVisible(1)
    if not voidp(tp) then
      tm = type_mems[tp]
      if not voidp(tm) then
        chain.setMember(tm)
        chain.setVisible(1)
      end if
    end if
  end if
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
