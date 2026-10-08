property ticketspots, foodspots, emptySpots, fullSpots, busspots, drinkspots, readyFood, foodReadyMS, timers, timeMultiplier, chef

on new me, Args
  foodReadyMS = Args.foodReadyMS
  ticketspots = []
  emptySpots = []
  fullSpots = [:]
  foodspots = []
  busspots = []
  drinkspots = []
  readyFood = []
  timers = [:]
  timeMultiplier = 1.0
  subscribe(me, [#hiliteAllDests, #unhiliteAllDests])
  chef = script("chef", "scripts.game").new()
  return me
end

on resetSpeed me
  foodReadyMS = getConfigData().foodReadyMS
end

on deleteSpots me
  call(#destroy, timers)
  timers.deleteAll()
  call(#destroy, drinkspots, 0)
  drinkspots.deleteAll()
  call(#destroy, foodspots, 0)
  foodspots.deleteAll()
  call(#destroy, ticketspots, 0)
  ticketspots.deleteAll()
  call(#destroy, busspots, 0)
  busspots.deleteAll()
  emptySpots.deleteAll()
end

on addSpot me, obj
  typ = obj.type
  if typ = #foodspot then
    me.addFoodSpot(obj)
  else
    if typ = #busspot then
      me.addBusSpot(obj)
    else
      if typ = #ticketspot then
        me.addTicketSpot(obj)
      else
        if typ = #drinkspot then
          me.addDrinkSpot(obj)
        end if
      end if
    end if
  end if
end

on hiliteAllDests me, typ
  if typ = #dishes then
    call(#setHilite, busspots, 1)
  else
    if typ = #ticket then
      call(#setHilite, ticketspots, 1)
    end if
  end if
end

on unhiliteAllDests me, typ
  if typ = #dishes then
    call(#setHilite, busspots, 0)
  else
    if typ = #ticket then
      call(#setHilite, ticketspots, 0)
    end if
  end if
end

on addBusSpot me, fs
  if busspots.getOne(fs) then
    return 
  end if
  busspots.add(fs)
  fs.setKitchen(me)
end

on setDrinkProps me, pl
  repeat with i in Keys(pl)
    repeat with d in drinkspots
      d[i] = pl[i]
    end repeat
  end repeat
end

on getMakingOrMade me, tbl
  if not voidp(timers.getaProp(tbl)) then
    return 1
  end if
  repeat with i in foodspots
    if i.food = tbl then
      return 1
    end if
  end repeat
end

on addDrinkSpot me, fs
  if drinkspots.getOne(fs) then
    return 
  end if
  drinkspots.add(fs)
  fs.setKitchen(me)
end

on addFoodSpot me, fs
  if foodspots.getOne(fs) then
    return 
  end if
  foodspots.add(fs)
  emptySpots.add(fs)
  fs.setKitchen(me)
end

on addTicketSpot me, ts
  if ticketspots.getOne(ts) then
    return 
  end if
  ticketspots.add(ts)
  ts.setKitchen(me)
end

on addTicket me, t
  if not voidp(timers.getaProp(t)) then
    return 
  end if
  ms = foodReadyMS * timeMultiplier
  if timers.count = 0 then
    chef.setVisible(1)
  end if
  tm = timers.getaProp(t)
  if not voidp(tm) then
    tm.destroy()
  end if
  timers.setaProp(t, timedCall(ms, [#target: me, #handler: #foodReady, #Args: t]))
end

on setTimeMultiplier me, num
  timeMultiplier = num
end

on foodReady me, t
  tm = timers.getaProp(t)
  if voidp(tm) then
    complain("debugging note for food disappearing bug")
    return 
  end if
  tm.destroy()
  if emptySpots.count = 0 then
    readyFood.add(t)
  else
    sp = emptySpots[1]
    emptySpots.deleteAt(1)
    fullSpots.setaProp(t, sp)
    sp.setFood(t)
  end if
  timers.deleteProp(t)
  if timers.count = 0 then
    chef.setVisible(0)
  end if
end

on killFood me, t
  tm = timers.getaProp(t)
  if not voidp(tm) then
    tm.destroy()
    timers.deleteProp(t)
  end if
  sp = fullSpots.getaProp(t)
  if not voidp(sp) then
    sp.setFood(VOID)
    me.addEmptySpot(sp)
  end if
  if timers.count = 0 then
    chef.setVisible(0)
  end if
  readyFood.deleteOne(t)
end

on returnFood me, t
  if not voidp(t.sitterGroup) then
    me.addTicket(t)
    t.setFoodReady(0)
  else
    tm = timers.getaProp(t)
    if not voidp(tm) then
      tm.destroy()
      timers.deleteProp(t)
    end if
  end if
  sp = fullSpots.getaProp(t)
  if timers.count then
    chef.setVisible(1)
  end if
  if voidp(sp) then
    return 
  end if
  fullSpots.deleteProp(t)
  sp.setFood(VOID)
  me.addEmptySpot(sp)
end

on addEmptySpot me, sp
  if readyFood.count then
    t = readyFood[1]
    readyFood.deleteAt(1)
    fullSpots.deleteOne(sp)
    fullSpots.setaProp(t, sp)
    sp.setFood(t)
  else
    if emptySpots.getOne(sp) = 0 then
      fullSpots.deleteOne(sp)
      emptySpots.add(sp)
    end if
  end if
end

on destroy me
  unsubscribe(me, #setDestHilite)
  room_mgr = VOID
  readyFood.deleteAll()
  me.deleteSpots()
end
