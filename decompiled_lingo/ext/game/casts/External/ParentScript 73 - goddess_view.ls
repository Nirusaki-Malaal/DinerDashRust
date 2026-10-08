property waiterobj, node, arms, bodynode, payloadnodes, foodnodes, currentPayloadNodes, room_mgr, errors, speed, checks, payloadmems, Tile, nextTile, direction, step, stageID, idleTimer, idleMSMin, idleMSRandom, floAnimData, mover
global glob

on new me, Args
  waiterobj = Args.waiter
  room_mgr = Args.room
  node = room_mgr.waiterLayer.newNode(getNodeData("flo_goddess"))
  bodynode = node["body"]
  node.setRegPoint(bodynode.getLoc())
  bodynode = node["body"]
  arms = node["arms"]
  payloadmems = [:]
  repeat with i in [#drinks, #ticket, #dishes, #somosa, #papadom, #crab, #bread]
    payloadmems[i] = member(string(i) & "_payload", "gfx.room")
  end repeat
  ap = [#loc]
  payloadnodes = []
  foodnodes = []
  repeat with i = 1 to 4
    payloadnodes.add(node["payload_" & i])
    foodnodes.add(node["food_" & i])
  end repeat
  node.setZOffset(15)
  call(#setAnimation, foodnodes, "goddess", [#loopCount: 1])
  currentPayloadNodes = []
  idleTimer = script("timeout").new()
  idleTimer.oneshot = 1
  idleTimer.callback = [#target: me, #handler: #startIdle]
  idleMSMin = 5000
  idleMSRandom = 15000
  mover = script("node_tween").new()
  return me
end

on destroy me
  node.destroy()
  idleTimer.destroy()
  voidRefs(me)
end

on updateAll me
  me.updatePayload()
end

on setRestaurant me, id
  node["head"].setAnimation("body_anim", [#loopCount: 0, #frameMS: 300])
  node["body"].setAnimation("head_anim", [#loopCount: 0, #frameMS: 300])
end

on updatePayload me
  pl = waiterobj.payload
  ct = pl.count
  call(#setVisible, foodnodes, 0)
  call(#setVisible, payloadnodes, 0)
  currentPayloadNodes.deleteAll()
  repeat with i = 1 to ct
    p = pl[i]
    obj = p.object
    tp = p.type
    if tp = #food then
      fn = foodnodes[i]
      m = member("tablenum_" & string(obj.tablenum), "gfx.numbers")
      nm = m.name
      if m.number <> -1 then
        fn["tablenum"].setMember(m)
      end if
      fn.setVisible(1)
      currentPayloadNodes.add(fn)
      next repeat
    end if
    pn = payloadnodes[i]
    pn.setVisible(1)
    if tp = #drinks then
      pn.setMember(member("drink_flo", "gfx." & room_mgr.currentStage))
    else
      pn.setMember(payloadmems[tp])
    end if
    currentPayloadNodes.add(pn)
  end repeat
  arms.setAnimation("carry_" & ct)
  if voidp(node["head"]) then
    return 
  end if
  node["head"].setAnimation("head_anim", [#loopCount: 0, #frameMS: 250])
  node["body"].setAnimation("body_anim", [#loopCount: 0, #frameMS: 175])
end

on showError me, e
end

on hideError me
end

on showCheckBonus me, num
  checks.setChain(0)
  checks.showBonus(num)
end

on setSpeed me, mspertile
  speed = integer(float(mspertile) / 3)
end

on moveToTile me, t
  nextTile = t
  if nextTile <> Tile then
    Dir = nextTile.pos - Tile.pos
  end if
  if voidp(t.node) then
    return 
  end if
  Args = [#node: node, #start: node.getLoc(), #end: t.node.getLocRel(Tile.node) + t.node.getRegPoint(), #ms: waiterobj.speed, #setter: #setLoc]
  mover.startTween(Args)
end

on startIdle me
end

on stopIdle me
  me.stop()
end

on stop me
end

on updatePayloadNodes
end

on setDirection me, Dir
  direction = Dir
  me.setTile(Tile)
end

on setTile me, t
  mover.endTween()
  Tile = t
  if voidp(t.node) then
    return 
  end if
  node.setParent(t.node)
  node.setLoc(t.node.getRegPoint())
  node.moveToFront()
end
