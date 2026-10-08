property waiterobj, node, legnode, bodynode, payloadnodes, foodnodes, currentPayloadNodes, room_mgr, errors, speed, checks, payloadmems, Tile, nextTile, direction, step, stageID, idleTimer, idleMSMin, idleMSRandom, floAnimData, podiumStarted
global glob

on new me, Args
  waiterobj = Args.waiter
  room_mgr = Args.room
  node = room_mgr.waiterLayer.newNode(getNodeData("flo"))
  bodynode = node["body"]
  node.setRegPoint(bodynode.getLoc() + point(0, 30))
  floAnimData = value(devoid(member("flos_anim_data").text))
  ad = floAnimData
  speed = 33
  legnode = node["legs"]
  bodynode = node["body"]
  node["body"].animator.setAnimData(ad.body)
  node["legs"].animator.setAnimData(ad.legs)
  mems = [#none: member("waiter", "gfx.people"), #left: member("flo_left", "gfx.people"), #both: member("flo_both", "gfx.people")]
  payloadmems = [:]
  repeat with i in [#drinks, #ticket, #dishes, #somosa, #papadom, #crab, #bread]
    payloadmems[i] = member(string(i) & "_payload", "gfx.room")
  end repeat
  ap = [#loc]
  payloadnodes = []
  foodnodes = []
  repeat with i = 1 to 2
    p = node["payload_" & i]
    p.setAnimData(ad.payload[i], ap)
    payloadnodes.add(p)
    f = node["food_" & i]
    f.setAnimData(ad.food[i], ap)
    foodnodes.add(f)
  end repeat
  currentPayloadNodes = []
  idleTimer = script("timeout").new()
  idleTimer.oneshot = 1
  idleTimer.callback = [#target: me, #handler: #startIdle]
  idleMSMin = 2000
  idleMSRandom = 4000
  return me
end

on setupFlo me, nm
  if not voidp(nm) then
    node.setDummyID(nm)
  end if
  bodynode = node["body"]
  node.setRegPoint(bodynode.getLoc() + point(0, 30))
  ad = floAnimData
  speed = 33
  bodynode = node["body"]
  if nm = "flo" then
    node["body"].animator.setAnimData(ad.body)
    legnode = node["legs"]
    node["legs"].animator.setAnimData(ad.legs)
  else
    legnode = node["head"]
    legnode.setAnimation("head_anim", [#frameMS: 200])
    bodynode.setAnimation("body_anim")
  end if
  mems = [#none: member("waiter", "gfx.people"), #left: member("flo_left", "gfx.people"), #both: member("flo_both", "gfx.people")]
  payloadmems = [:]
  repeat with i in [#drinks, #ticket, #dishes, #somosa, #papadom, #crab, #bread]
    payloadmems[i] = member(string(i) & "_payload", "gfx.room")
  end repeat
  ap = [#loc]
  payloadnodes = []
  foodnodes = []
  repeat with i = 1 to 2
    p = node["payload_" & i]
    p.setAnimData(ad.payload[i], ap)
    payloadnodes.add(p)
    f = node["food_" & i]
    f.setAnimData(ad.food[i], ap)
    foodnodes.add(f)
  end repeat
end

on destroy me
  bodynode = VOID
  node.destroy()
  node = VOID
  waiterobj = VOID
  payloadnodes.deleteAll()
  foodnodes.deleteAll()
  idleTimer.destroy()
  idleTimer = VOID
end

on updateAll me
  me.updatePayload()
end

on setRestaurant me, id
  was_g = string(stageID) = "goddess"
  if stageID = id then
    return 
  end if
  stageID = id
  if id = #goddess then
    me.setupFlo("flo_goddess")
  else
    if was_g then
      me.setupFlo("flo")
    end if
  end if
  call(#setAnimation, foodnodes, string(id), [#animProps: [#loc, #member], #loopCount: 1])
end

on updatePayload me
  pl = waiterobj.payload
  ct = pl.count
  legnode.setVisible(1)
  call(#setVisible, foodnodes, 0)
  call(#setVisible, payloadnodes, 0)
  currentPayloadNodes.deleteAll()
  if direction = "up" then
    direction = "down"
  end if
  repeat with i = 1 to ct
    p = pl[i]
    obj = p.object
    tp = p.type
    if tp = #food then
      fn = foodnodes[i]
      m = member("tablenum_" & obj.tablenum, "gfx.numbers")
      if m.number <> -1 then
        fn["tablenum"].setMember(m)
        fn.setVisible(1)
      end if
      fn.setAnimation(direction & "_stop", [#animProps: [#loc], #loopCount: 1])
      currentPayloadNodes.add(fn)
      next repeat
    end if
    pn = payloadnodes[i]
    pn.setVisible(1)
    pn.setAnimation(direction & "_stop", [#animProps: [#loc], #loopCount: 1])
    if tp = #drinks then
      pn.setMember(member("drink_flo", "gfx." & room_mgr.currentStage))
    else
      pn.setMember(payloadmems[tp])
    end if
    currentPayloadNodes.add(pn)
  end repeat
  if ct = 0 then
    bodynode.setAnimation(direction & "_stop")
    idleTimer.setPeriod(idleMSMin + random(idleMSRandom))
    idleTimer.start()
  else
    if ct = 1 then
      bodynode.setAnimation(direction & "_carry_1_stop", [#loopCount: 1])
    else
      bodynode.setAnimation(direction & "_carry_2_stop", [#loopCount: 1])
    end if
  end if
  call(#setAnimation, currentPayloadNodes, direction & "_stop", [#animProps: [#loc]])
end

on showError me, e
end

on hideError me
end

on updateChain me
  if voidp(waiterobj) then
    return 
  end if
end

on showCheckBonus me, num
  checks.setChain(0)
  checks.showBonus(num)
end

on setSpeed me, mspertile
  speed = integer(float(mspertile) / 3)
end

on moveToTile me, t
  if podiumStarted then
    me.stopIdle()
  end if
  idleTimer.stop()
  nextTile = t
  if nextTile <> Tile then
    Dir = nextTile.pos - Tile.pos
  end if
  if Dir[1] then
    if Dir[1] > 0 then
      direction = "right"
    else
      direction = "left"
    end if
  else
    if Dir[2] then
      if Dir[2] > 0 then
        direction = "down"
      else
        direction = "up"
      end if
    else
      if voidp(direction) then
        direction = "down"
      end if
    end if
  end if
  speed = integer(float(waiterobj.speed) / 3)
  inf = [#loopCount: 1, #frameMS: speed]
  pinf = [#loopCount: 1, #frameMS: speed, #animProps: [#loc]]
  step = not step
  legnode.setVisible(1)
  legnode.setAnimation(direction & "_" & step, inf)
  ct = waiterobj.payload.count
  if ct = 0 then
    bodynode.setAnimation(direction & "_" & step, inf)
  else
    if ct = 1 then
      bodynode.setAnimation(direction & "_carry_1", inf)
      currentPayloadNodes[1].setAnimation(direction, pinf)
    else
      bodynode.setAnimation(direction & "_carry_2", inf)
      call(#setAnimation, currentPayloadNodes, direction, pinf)
    end if
  end if
end

on startIdle me
  if waiterobj.payload.count then
    return 
  end if
  legnode.setVisible(0)
  r = random(4)
  if r = 1 then
    ct = 1
    frameMS = 150
  else
    if r = 2 then
      ct = 1 + random(2)
      frameMS = 100
    else
      r = 3
      ct = random(5)
    end if
  end if
  bodynode.setAnimation("idle_" & r, [#loopCount: ct, #frameMS: frameMS, #callback: [#target: me, #handler: #stopIdle]])
end

on stopIdle me
  podiumStarted = 0
  legnode.setVisible(1)
  me.updatePayload()
  me.stop()
end

on stop me
  legnode.setVisible(1)
  legnode.setAnimation(direction & "_stop", [#loopCount: 1])
  ct = waiterobj.payload.count
  if ct = 0 then
    bodynode.setAnimation(direction & "_stop", [#loopCount: 1])
    idleTimer.setPeriod(idleMSMin + random(idleMSRandom))
    idleTimer.start()
  else
    if ct = 1 then
      bodynode.setAnimation(direction & "_carry_1_stop", [#loopCount: 1])
    else
      bodynode.setAnimation(direction & "_carry_2_stop", [#loopCount: 1])
    end if
  end if
  call(#setAnimation, currentPayloadNodes, direction & "_stop", [#loopCount: 1, #animProps: [#loc]])
end

on updatePayloadNodes
end

on startPodium me
  podiumStarted = 1
  bodynode.animator.stop()
  idleTimer.stop()
  legnode.setVisible(0)
  bodynode.setAnimation("podium_1", [#loopCount: 1])
  bodynode.queueAnimation("podium_2", [#loopCount: 0])
  call(#setVisible, foodnodes, 0)
  call(#setVisible, payloadnodes, 0)
end

on finishPodium me
  bodynode.setAnimation("podium_3", [#loopCount: 1, #callback: [#target: me, #handler: #stopIdle]])
end

on setDirection me, Dir
  direction = Dir
  me.setTile(Tile)
end

on setTile me, t
  Tile = t
  if voidp(t.node) then
    return 
  end if
  if direction = "up" then
    direction = "down"
  end if
  node.setParent(t.node)
  node.setLoc(t.node.getRegPoint())
  node.moveToFront()
  me.stop()
end
