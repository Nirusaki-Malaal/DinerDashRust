property room_mgr, game_mgr, destqueue, currentDest, maxQueue, chain, minChain, pickupChains, dropChains, chainMode, maxChain, dropPoints, pickupPoints, payload, payloadTypes, tilePath, currentTile, destTile, maxPayload, views, speed, moveTimer, room, currentObj, pickupSounds, dropSounds, snackTypes, stageID

on new me, Args
  cfg = getConfigData()
  minChain = cfg.minChain
  destqueue = []
  maxQueue = 5
  checkchain = []
  maxChecks = 8
  actionPoints = cfg.actionPoints
  chain = []
  pickupChains = [#ticket, #dishes]
  dropChains = [#food, #check]
  pickupPoints = [:]
  dropPoints = [:]
  repeat with i in [#ticket, #dishes]
    pickupPoints[i] = actionPoints[i]
  end repeat
  repeat with i in [#food, #check, #drinks, #somosa, #papadom, #bread, #crab]
    dropPoints[i] = actionPoints[i]
  end repeat
  maxChain = 12
  tilePath = []
  room = Args.room_manager
  game_mgr = Args.game_manager
  pickupSounds = [:]
  pickupSounds[#drinks] = "sfx_get_drinks_2"
  pickupSounds[#food] = "sfx_pickup_food_1"
  pickupSounds[#dishes] = "sfx_clear_dishes_1"
  pickupSounds[#ticket] = "sfx_take_order_1"
  dropSounds = [:]
  dropSounds[#drinks] = "sfx_dropoff_drinks_1"
  dropSounds[#food] = "sfx_deliver_food_2"
  dropSounds[#dishes] = "sfx_dish_dropoff_2"
  dropSounds[#ticket] = "sfx_deliver_order_1"
  dropSounds[#check] = "sfx_menu_down_2"
  snackTypes = [#crab, #somosa, #papadom, #bread]
  payload = []
  payloadTypes = []
  maxPayload = 2
  views = []
  views.add(script("waiter_view").new([#waiter: me, #room: Args.room_manager]))
  speed = getConfigData().waiterSpeedMS
  subscribe(me, #keyDown)
  return me
end

on resetSpeed me
  speed = getConfigData().waiterSpeedMS
end

on destroy me
  call(#destroy, views)
  views.deleteAll()
  if not voidp(moveTimer) then
    moveTimer.destroy()
    moveTimer = VOID
  end if
  unsubscribe(me, #keyDown)
  room_mgr = VOID
  game_mgr = VOID
  call(#destroy, destqueue)
  destqueue.deleteAll()
end

on queueDest me, dest
  if destqueue.count >= maxQueue then
    return 
  end if
  dest.setSelected(1)
  if (destqueue.count = 0) and voidp(currentDest) then
    me.setDest(dest)
    call(#updateQueue, views)
    return 
  end if
  destqueue.add(dest)
  call(#updateQueue, views)
end

on reachDest me
  if voidp(currentDest) then
    return 
  end if
  t = currentDest.getTile()
  diff = currentTile.pos - t.pos
  if diff[1] > 0 then
    Dir = "left"
  else
    if diff[1] < 0 then
      Dir = "right"
    else
      if diff[2] > 0 then
        Dir = "up"
      else
        Dir = "down"
      end if
    end if
  end if
  call(#setDirection, views, Dir)
  err = currentDest.doFloAction(me)
  currentDest.setSelected(0)
  currentObj = currentDest
  currentDest = VOID
  if err = 0 then
    err = #nothing_to_do
  end if
  if err <> #Ok then
    me.clearQueue()
    call(#showError, views, err)
    call(#updateQueue, views)
  else
    call(#hideError, views)
    me.nextDest()
  end if
end

on nextDest me
  if destqueue.count then
    d = destqueue[1]
    destqueue.deleteAt(1)
    me.setDest(d)
  end if
  call(#updateQueue, views)
end

on setDest me, obj
  if voidp(obj) then
    return 
  end if
  if not voidp(currentObj) then
    if currentObj[#type] = #podium then
      currentObj.stop()
    end if
  end if
  currentDest = obj
  if not voidp(currentDest) then
    if currentDest.getTile() = currentTile then
      me.reachDest()
      return 
    end if
  end if
  dtile = obj.getTile()
  p = findPath(room.grid, currentTile.pos, dtile.pos, [#nextto: 1])
  tilePath.deleteAll()
  if voidp(p) then
    me.clearQueue()
    return 
  end if
  repeat with i in p
    if i <> currentTile.pos then
      tilePath.add(room.grid.getTileAt(i))
    end if
  end repeat
  call(#hideError, views)
  if tilePath.count then
    me.moveTo(tilePath[1])
  else
    me.reachDest()
  end if
end

on reset me
  payload.deleteAll()
  chain.deleteAll()
  destqueue.deleteAll()
  tilePath.deleteAll()
  currentDest = VOID
  currentTile = VOID
  if not voidp(moveTimer) then
    moveTimer.stop()
  end if
  call(#updateAll, views)
end

on clearChain me
  chain.deleteAll()
end

on clearQueue me
  if not voidp(currentDest) then
    destqueue.add(currentDest)
  end if
  call(#setSelected, destqueue, 0)
  destqueue.deleteAll()
  call(#updateQueue, views)
end

on canPickUp me, typ
  return payload.count < maxPayload
end

on hasPayloadType me, typ
  return payloadTypes.getOne(typ) <> 0
end

on hasPayload me, typ, arg
  repeat with i in payload
    if i.type = typ then
      if (typ = #drinks) or snackTypes.getOne(typ) or (i.object = arg) then
        return 1
      end if
    end if
  end repeat
  return 0
end

on getPayload me, typ
  ls = []
  repeat with i in payload
    if i.type = typ then
      ls.add(i.object)
    end if
  end repeat
  return ls
end

on refreshPayloadTypes me
  payloadTypes.deleteAll()
  repeat with i in payload
    t = i.type
    if payloadTypes.getOne(t) = 0 then
      payloadTypes.add(t)
    end if
  end repeat
end

on setRestaurant me, stid
  was_g = string(stageID) = "goddess"
  if string(stid) = "goddess" then
    maxPayload = 4
  else
    maxPayload = 2
  end if
  if string(stageID) = string(stid) then
    return 
  end if
  stageID = stid
  if was_g then
    call(#destroy, views)
    views.deleteAll()
    views.add(script("waiter_view").new([#waiter: me, #room: room]))
  else
    if string(stid) = "goddess" then
      call(#destroy, views)
      views.deleteAll()
      views.add(script("goddess_view").new([#waiter: me, #room: room]))
    end if
  end if
  call(#setRestaurant, views, stid)
end

on pickUp me, typ, arg
  call(#stopIdle, views)
  if payload.count < maxPayload then
    if not voidp(arg) and me.hasPayload(typ, arg) then
      return 0
    end if
    if not voidp(arg) then
      if (arg.type = #table) and (typ = #food) then
        arg.setFoodPickedUp(1)
      end if
    end if
    sendEvent(#hiliteAllDests, typ)
    payload.add([#type: typ, #object: arg])
    if payloadTypes.getOne(typ) = 0 then
      payloadTypes.add(typ)
    end if
    call(#updatePayload, views)
    if typ <> chainMode then
      me.clearChain()
      chainMode = typ
    end if
    if pickupChains.getOne(typ) then
      if chain.getOne(arg) = 0 then
        chain.add(arg)
      end if
    end if
    if not voidp(arg) then
      pts = pickupPoints[typ]
      if not voidp(pts) then
        Args = [:]
        ct = chain.count
        if ct >= minChain then
          Args[#multiplier] = ct
          Args[#type] = chainMode
          pts = pts * ct
        end if
        Args[#score] = pts
        arg.getBonus(Args)
      end if
    end if
    sfx = pickupSounds[typ]
    if not voidp(sfx) then
      SndSFX(sfx)
    end if
    return 1
  end if
end

on dropCheck me, tbl
  if chainMode <> #check then
    me.clearChain()
    chainMode = #check
  end if
  if chain.getOne(tbl) = 0 then
    chain.add(tbl)
  end if
  pts = dropPoints.check
  Args = [:]
  ct = chain.count
  if ct >= minChain then
    Args[#multiplier] = ct
    Args[#type] = #check
    pts = pts * ct
  end if
  Args[#score] = pts
  tbl.getBonus(Args)
  sfx = dropSounds[#check]
  if not voidp(sfx) then
    SndSFX(sfx)
  end if
end

on setSpeed me, ms
  call(#setSpeed, views, ms)
  speed = ms
end

on dropOff me, typ, arg
  obj = arg
  if typ = #drinks then
    repeat with i = 1 to payload.count
      p = payload[i]
      if p.type = #drinks then
        arg.haveDrink(p.object.moodPoints)
        payload.deleteAt(i)
        exit repeat
      end if
    end repeat
  else
    if snackTypes.getOne(typ) then
      succ = 0
      repeat with i = 1 to payload.count
        p = payload[i]
        if p.type = typ then
          payload.deleteAt(i)
          succ = 1
          exit repeat
        end if
      end repeat
      if not succ then
        return 0
      end if
    else
      if not payload.deleteOne([#type: typ, #object: obj]) and (typ <> #check) then
        return 0
      end if
    end if
  end if
  me.refreshPayloadTypes()
  if payloadTypes.getOne(typ) = 0 then
    sendEvent(#unhiliteAllDests, typ)
  end if
  call(#updatePayload, views)
  if typ <> chainMode then
    me.clearChain()
    chainMode = typ
  end if
  if dropChains.getOne(typ) then
    if chain.getOne(arg) = 0 then
      chain.add(arg)
    end if
  end if
  if not voidp(arg) then
    pts = dropPoints[typ]
    if not voidp(pts) then
      Args = [:]
      ct = chain.count
      if ct >= minChain then
        Args[#multiplier] = ct
        Args[#type] = typ
        pts = pts * ct
      end if
      Args[#score] = pts
      if (ilk(arg) = #instance) and (arg[#type] = #table) then
        arg.getBonus(Args)
      end if
    end if
  end if
  sfx = dropSounds[typ]
  if not voidp(sfx) then
    SndSFX(sfx)
  end if
  return 1
end

on dropOffAll me, typ
  if payloadTypes.getOne(typ) = 0 then
    return 0
  end if
  repeat with i in payload.duplicate()
    if i.type = typ then
      payload.deleteOne(i)
    end if
  end repeat
  sendEvent(#unhiliteAllDests, typ)
  sfx = dropSounds[typ]
  if not voidp(sfx) then
    SndSFX(sfx)
  end if
  call(#updatePayload, views)
  me.refreshPayloadTypes()
  return 1
end

on arriveAtTile me, t
  currentTile = t
  if tilePath.count then
    tilePath.deleteAt(1)
  end if
  call(#setTile, views, t)
  if tilePath.count then
    me.moveTo(tilePath[1])
  else
    me.reachDest()
  end if
end

on keyDown me, k
  if k = SPACE then
    me.stopMoving()
  end if
end

on stopMoving me
  if not voidp(moveTimer) then
    moveTimer.stop()
  end if
  me.clearQueue()
  if not voidp(currentDest) then
    currentDest.setSelected(0)
    currentDest = VOID
  end if
  tilePath.deleteAll()
end

on killFood me, tbl
  if payloadTypes.getOne(#food) = 0 then
    return 
  end if
  me.dropOff(#food, tbl)
end

on moveTo me, Tile
  if Tile = currentTile then
    me.arriveAtTile(Tile)
  end if
  moveTimer = timedCall(speed, [#target: me, #handler: #arriveAtTile, #Args: Tile])
  call(#moveToTile, views, Tile)
end

on groupLeft me, g
end
