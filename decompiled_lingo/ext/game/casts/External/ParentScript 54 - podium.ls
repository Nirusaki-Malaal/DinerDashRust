property node, heart, chargeMS, delayms, activeMS, moodPoints, hearts, game_mgr, room_mgr, check, floHere, ready, Tile, destTile, type, timer, shapeinfo
global glob

on new me, Args
  type = #podium
  node = Args.node
  node.setVisible(1)
  shapeinfo = [[me]]
  game_mgr = Args.game_mgr
  room_mgr = Args.room_mgr
  if ilk(Args[#config], #propList) then
    me.setConfig(Args.config)
  end if
  node = Args.node
  node.addMouseListener(me)
  heart = node["podium_heart"]
  heart.setVisible(0)
  check = node["check"]
  check.setVisible(0)
  pscript = script("podium_heart", "scripts.game")
  hearts = []
  repeat with i = 1 to 10
    hearts.add(pscript.new(node["heart_" & i]))
  end repeat
  ready = 1
  return me
end

on setConfig me, cfg
  repeat with i = 1 to cfg.count
    k = cfg.getPropAt(i)
    me.setProp(k, cfg[i])
  end repeat
end

on clearTimer me
  if not voidp(timer) then
    timer.destroy()
    timer = VOID
  end if
end

on setTile me, t
  Tile = t
end

on setDestTile me, t
  destTile = t
end

on getTile me
  if not voidp(destTile) then
    return destTile
  end if
  return Tile
end

on doFloAction me, flo
  if floHere then
    return 
  end if
  floHere = 1
  if not ready then
    return 
  end if
  flo.clearChain()
  me.startCharging()
  return #Ok
end

on setReady me, r
  if voidp(heart) then
    return 
  end if
  ready = r
  call(#setVisible, hearts, r)
  if ready and floHere then
    me.startCharging()
  end if
end

on startCharging me
  if not ready then
    return 
  end if
  if not floHere then
    me.stop()
    return 
  end if
  call(#startPodium, room_mgr.waiter.views)
  heart.setVisible(1)
  heart.setAnimation("grow", [#frameMS: chargeMS / 3, #loopCount: 1, #callback: [#target: me, #handler: #Activate]])
end

on Activate me
  if voidp(heart) then
    return 
  end if
  call(#fire, hearts)
  heart.setAnimation("grow_full")
  ready = 0
  me.clearTimer()
  timer = timedCall(activeMS, [#target: me, #handler: #recharge])
  game_mgr.activatePodium(moodPoints)
  call(#finishPodium, room_mgr.waiter.views)
end

on recharge me
  if voidp(heart) then
    return 
  end if
  call(#stop, hearts)
  call(#setVisible, hearts, 0)
  heart.setVisible(0)
  me.clearTimer()
  timer = timedCall(delayms, [#target: me, #handler: #setReady, #Args: 1])
end

on stop me
  if voidp(heart) then
    return 
  end if
  floHere = 0
  call(#stop, hearts)
  call(#setVisible, hearts, ready)
  heart.setVisible(0)
  heart.animator.stop()
end

on setSelected me, s
  check.setVisible(s)
end

on mouseEnter me
  setcursor(#finger_up)
end

on mouseLeave me
  setcursor(#arrow)
end

on mouseDown me
  setcursor(#finger_down)
  if not floHere then
    room_mgr.selectDest(me)
  end if
end

on mouseUp me
  setcursor(#finger_up)
end

on destroy me
  me.clearTimer()
  floHere = 0
  game_mgr = VOID
  call(#destroy, hearts)
  hearts.deleteAll()
  node.removeMouseListener(me)
  node = VOID
  heart = VOID
  faketile = VOID
  check = VOID
end
