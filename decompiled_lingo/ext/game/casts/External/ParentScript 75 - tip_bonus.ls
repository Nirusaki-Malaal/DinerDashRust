property numbermems, node, numnode, numbers, extrastar, coin, homes, locs, showing, home_yreg, speed, height, coinmems, coinheight, coinms, counter, coincounter, coinspeed, alpha, bouncems, view_mgr, type, phase, numwidth
global glob

on new me
  node = glob.managers.game.node.newNode(getNodeData("tip_new"))
  view_mgr = glob.managers.view
  type = "tip"
  numnode = node["number"]
  extrastar = node["extrastar"]
  coin = node["coin"]
  homes = [#star: extrastar.getLoc(), #coin: coin.getLoc(), #num: numnode.getYReg()]
  locs = homes.duplicate()
  numbers = []
  repeat with i = 1 to 5
    numbers.add(numnode["number_" & i])
  end repeat
  numwidth = numbers[1].getWidth()
  numbermems = []
  repeat with num = 0 to 9
    numbermems.add(member("green." & num, "gfx.numbers"))
  end repeat
  coinmems = []
  repeat with i = 1 to 4
    coinmems.add(member("coin_flip_" & i, "gfx.screens"))
  end repeat
  node.setZOffset(1000000)
  node.setRegPoint(numbers[3].getLoc() + [0, 10])
  call(#setVisible, node.children, 0)
  coinspeed = 400.0
  bouncems = 200.0
  coinheight = 60
  home_yreg = numnode.getYReg()
  speed = 30.0
  height = 30.0
  node.setVisible(1)
  return me
end

on show me, Args
  num = Args.total
  if not num.integerp then
    return 
  end if
  if (num > 99999) or (num < 0) then
    return 
  end if
  str = string(num)
  ct = str.char.count
  repeat with i = 1 to 5
    n = numbers[i]
    if i > ct then
      n.setVisible(0)
      next repeat
    end if
    n.setMember(numbermems[integer(str.char[i]) + 1])
    n.setVisible(1)
  end repeat
  numnode.setXReg(numwidth * ct * 0.5)
  phase = 1
  coin.setLoc(homes.coin)
  coin.setVisible(1)
  counter = 0
  coinms = 0
  numnode.setYReg(homes.num)
  coin.setAnimation("flip", [#loopCount: 10, #frameMS: 30, #group: "score", #animProps: [#member]])
  subscribe(me, #exitFrame, "score")
end

on hide me
  call(#setVisible, node.children, 0)
  unsubscribe(me, #exitFrame, "score")
  view_mgr.returnView(me)
end

on setObject me
end

on removeObject me
end

on exitFrame me, ms
  if node.voidp then
    return 
  end if
  if phase = 1 then
    coinms = coinms + ms
    if coinms > coinspeed then
      phase = 2
      coin.setVisible(0)
      coinms = 0
      numnode.setVisible(1)
    else
      counter = (counter mod 4) + 1
      coin.setLoc(homes.coin - point(0, sin(coinms / coinspeed * PI * 0.80000000000000004) * coinheight))
    end if
  else
    if phase = 2 then
      coinms = coinms + ms
      counter = counter + (ms * 0.001 * speed)
      if counter > height then
        me.hide()
        return 
      end if
      if coinms > bouncems then
        coinms = 0
        phase = 3
      else
        numnode.setYReg(homes.num + (abs(sin(coinms / bouncems * PI)) * 10))
      end if
    else
      if phase = 3 then
        coinms = coinms + ms
        if coinms > bouncems then
          me.hide()
          phase = 0
        end if
      end if
    end if
  end if
end

on destroy me
  unsubscribe(me, #exitFrame, "score")
  node.destroy()
  repeat with i = 1 to me.count
    me[i] = VOID
  end repeat
end
