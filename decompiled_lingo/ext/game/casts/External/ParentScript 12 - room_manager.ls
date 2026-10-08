property glob, game_mgr, grid, currentLevel, tableChains, minTableChainCount, sitterGroups, waiter, kitchen, tables, bar, stations, node, stationNodes, foodnodes, podiumNode, snackNodes, snackStations, timeMultiplier, podium, bgconfig, levelLoaded, currentStage, obstacles, mime, mimeNode, bgnode, closingSign, closingHome, closingOffscreen, tweener, disappearNodes, appearNodes, previousItems, previousTables, previousSnacks, bgstate, tableLayer, waiterLayer, gridLayer

on new me, g, n
  glob = g
  game_mgr = glob.managers.game
  node = n
  tableLayer = n.newNode()
  waiterLayer = n.parent.newNode()
  bgconfig = glob.managers.config.gfx
  sitterGroups = []
  tables = []
  tableChains = []
  minTableChainCount = 2
  obstacles = []
  stations = [:]
  waiter = script("waiter", "scripts.game").new([#room_manager: me, #game_manager: game_mgr])
  foodnodes = []
  tweener = script("node_tween").new()
  disappearNodes = []
  appearNodes = []
  previousItems = []
  previousTables = []
  previousSnacks = []
  stationNodes = [:]
  snackNodes = []
  snackStations = [:]
  timeMultiplier = 1.0
  kitchen = script("kitchen", "scripts.game").new([#foodReadyMS: getConfigData().foodReadyMS])
  grid = script("grid_manager").new([#room_manager: me])
  return me
end

on destroy me
  me.destroyLevel()
  if not voidp(waiter) then
    waiter.destroy()
    waiter = VOID
  end if
  if not voidp(kitchen) then
    kitchen.destroy()
  end if
  if not voidp(grid) then
    grid.destroy()
  end if
  voidRefs(me)
end

on getMorePaying me
  ret = 0
  repeat with i in sitterGroups
    s = i.getState()
    if (s = #eating) or (s = #check) then
      ret = 1
    end if
  end repeat
  return ret
end

on updateBGState me
  st = bgconfig[currentStage]
  if voidp(st) then
    return 
  end if
  if st.states.count then
    if (game_mgr.playMode = #endless) and not voidp(glob.managers[#savegame]) then
      bgs = st.states[glob.managers.savegame.getUnlockedBG(currentStage)]
    else
      bgs = st.states[1]
    end if
    repeat with i in Keys(bgs)
      n = node["bg"][i]
      if not voidp(n) then
        node["bg"][i].setVisible(bgs[i])
      end if
    end repeat
  end if
end

on resetObjects me
  waiter.resetSpeed()
  kitchen.resetSpeed()
end

on loadStage me, stid
  if levelLoaded then
    complain("loadStage must be called between destroyLevel and loadLevel")
    return 
  end if
  currentStage = stid
  waiter.setRestaurant(stid)
  st = bgconfig[stid]
  if voidp(st) then
    complain("no stage with id of" && st)
    return 
  end if
  bg = node["bg"]
  bg.setDummyID(st.nodeID)
  kitchen.chef.setNode(bg["chef"])
  node.moveToBack()
  game_mgr.interface_node.moveToFront()
  podiumNode = bg["podium"]
  if not voidp(podiumNode) then
    podiumNode.setVisible(0)
    podiumNode.setZOffset(400000)
  end if
  mimeNode = bg["mime"]
  if not voidp(mimeNode) then
    mimeNode.setVisible(0)
    mimeNode.moveToFront()
    mimeNode.setZOffset(400000)
  end if
  tweener.endTween()
  closingSign = bg["closing_sign"]
  closingHome = closingSign.getLoc()
  closingSign.setLoc(closingHome + point(-200, 0))
  foodnodes.deleteAll()
  repeat with i = 1 to 5
    fn = bg["food_" & i]
    if voidp(fn) then
      exit repeat
    end if
    foodnodes.add(fn)
  end repeat
  call(#setVisible, foodnodes, 0)
  call(#destroy, snackNodes)
  stationNodes.deleteAll()
  repeat with i in ["drinkspot", "busspot", "ticketspot"]
    n = bg[i]
    if voidp(i) then
      next repeat
    end if
    stationNodes[symbol(i)] = n
  end repeat
  call(#setVisible, stationNodes, 0)
  snackNodes.deleteAll()
  repeat with i = 1 to 3
    sn = bg["snacks_" & i]
    if not voidp(sn) then
      snackNodes.add(sn)
    end if
  end repeat
  call(#setVisible, snackNodes, 0)
  call(#setZOffset, snackNodes, 1)
end

on destroyLevel me
  call(#destroy, sitterGroups)
  sitterGroups.deleteAll()
  if not voidp(waiter) then
    waiter.reset()
  end if
  tweener.endTween()
  if not voidp(bar) then
    bar.destroy()
    bar = VOID
  end if
  if not voidp(podium) then
    podium.destroy()
    podium = VOID
  end if
  if not voidp(mime) then
    mime.destroy()
    mime = VOID
  end if
  call(#destroy, obstacles)
  obstacles.deleteAll()
  grid.destroyGrid()
  if not voidp(kitchen) then
    kitchen.deleteSpots()
  end if
  call(#destroy, tables)
  tables.deleteAll()
  repeat with i in stations
    call(#destroy, i)
  end repeat
  stations.deleteAll()
  levelLoaded = 0
end

on setTimeMultiplier me, num
  timeMultiplier = num
  kitchen.setTimeMultiplier(num)
end

on groupSeated me, sgroup, tbl
  if sitterGroups.getOne(sgroup) <> 0 then
    complain("tried to seat a group already sitting")
    return 
  end if
  sitterGroups.add(sgroup)
end

on closingTime me
  if voidp(closingSign) then
    return 
  end if
  tweener.startTween([#node: closingSign, #start: closingSign.getLoc(), #end: closingHome, #setter: #setLoc, #ms: 750])
end

on numWantSnack me, typ
  num = 0
  repeat with i in sitterGroups
    num = num + (i.wantedSnack = typ)
  end repeat
end

on groupLeft me, sg, tbl
  if sitterGroups.getOne(sg) = 0 then
    complain("sitter group not seated tried to leave")
    return 
  end if
  kitchen.killFood(tbl)
  waiter.killFood(tbl)
  sitterGroups.deleteOne(sg)
  sg.destroy()
  game_mgr.checkLevelDone()
end

on selectDest me, obj
  waiter.queueDest(obj)
  if (obj.type <> #podium) and not voidp(podium) then
    podium.stop()
  end if
end

on removeGroup me, sg
  if sitterGroups.getOne(sg) <> 0 then
    sitterGroups.deleteOne(sg)
  end if
end

on setStereo me, dat
end

on setOven me, dat
end

on addTable me, tdata
  pos = tdata.chairs[1].pos
  t = script("table", "scripts.game").new([#room_manager: me, #capacity: tdata.chairs.count, #parentNode: tableLayer, #stage: currentStage, #tablenum: tables.count + 1, #data: tdata])
  if grid.placeObject(t, tdata.chairs[1].pos - [0, 1]) then
    tables.add(t)
    destTile = grid.getTileAt(pos[1] + 2, pos[2] - 2)
    t.setDestTile(destTile)
    grid.updateLocZ()
  else
    complain("couldn't place table")
    t.destroy()
  end if
end

on setPodiumProps me, p
  if voidp(p) then
    podiumNode.setVisible(0)
    podium.setReady(0)
    return 
  end if
  if voidp(podium) then
    podium = script("podium", "scripts.game").new([#node: podiumNode, #game_mgr: game_mgr, #room_mgr: me])
    t = grid.getTileNearest(podiumNode.getLoc())
    if not grid.placeObject(podium, t.pos) then
      complain("couldn't place podium")
      podium.destroy()
      return 
    end if
    podium.setDestTile(grid.getTileAt(t.pos[1], t.pos[2] - 1))
  end if
  podium.setConfig(p)
  podiumNode.setVisible(1)
  podium.setReady(1)
end

on hasNewItems me
  return appearNodes.count or disappearNodes.count
end

on flashItems me
  call(#setVisible, appearNodes, 0)
  call(#setVisible, disappearNodes, 1)
  repeat with i in appearNodes
    flashNode([#node: i, #loopCount: 5, #ms: 200, #group: "room"])
  end repeat
  repeat with i in disappearNodes
    flashNode([#node: i, #loopCount: 5, #ms: 200, #group: "room", #invert: 1])
  end repeat
end

on setDrinkProps me, p
  if voidp(stations[#drinkspot]) then
    n = stationNodes[#drinkspot]
    t = grid.getTileNearest(n.getLoc())
    if (t.occupant.type = #empty) or (t.occupant.type = #Full) then
      obj = script("drinkspot").new([#node: n, #room_mgr: me, #timeMultiplier: timeMultiplier, #props: p])
      if not grid.placeObject(obj, t.pos) then
        complain("couldn't place drinks")
        obj.destroy()
        return 
      end if
      obj.setDestTile(grid.getTileAt(t.pos[1], t.pos[2] + 2))
      kitchen.addSpot(obj)
      stations[#drinkspot] = obj
    else
      complain("couldn't place drinks")
      return 
    end if
  end if
  repeat with i in Keys(p)
    stations.drinkspot[i] = p[i]
  end repeat
end

on setMimeProps me, props
  if not voidp(mime) then
    mime.destroy()
  end if
  if not voidp(mimeNode) then
    if voidp(props) then
      props = [:]
    end if
    props[#node] = mimeNode
    mime = script("mime").new(props)
  end if
end

on setSnacks me, dat, pc
  mySnacks = previousSnacks.duplicate()
  previousSnacks.deleteAll()
  if dat.voidp then
    dat = []
  end if
  scr = script("snackspot", "scripts.game")
  call(#destroy, snackStations)
  snackStations.deleteAll()
  repeat with i = 1 to snackNodes.count
    sn = snackNodes[i]
    if i > dat.count then
      sn.setVisible(0)
      next repeat
    end if
    stype = dat[i]
    t = grid.getTileNearest(sn.getLocRel())
    dt = grid.getTileAt(t.pos[1], t.pos[2])
    if (t.occupant.type = #empty) or (t.occupant.type = #Full) then
      obj = scr.new([#node: sn, #type: stype, #room_mgr: me])
      if mySnacks.getOne(stype) = 0 then
        appearNodes.add(sn)
      end if
      obj.setDestTile(dt)
      previousSnacks.add(stype)
      snackStations[stype] = obj
    end if
  end repeat
  call(#setSnacks, sitterGroups, dat, pc)
end

on loadLevel me, lvl_all
  if not voidp(mimeNode) then
    mimeNode.setVisible(0)
  end if
  disappearNodes.deleteAll()
  appearNodes.deleteAll()
  myItems = previousItems.duplicate()
  myTables = previousTables.duplicate()
  previousTables.deleteAll()
  previousItems.deleteAll()
  tweener.endTween()
  if not voidp(closingSign) then
    closingSign.setLoc(closingHome - point(200, 0))
  end if
  lvl = lvl_all[#layout]
  if voidp(lvl) then
    lvl = getLevelData()
  end if
  me.destroyLevel()
  currentLevel = lvl
  grid.setSize([18, lvl.height])
  if currentStage = #finedining then
    grid.blockColumn()
  end if
  bgstate_name = lvl_all[#bgstate]
  if not voidp(bgstate_name) then
    old_bgstate = bgstate
    bgstate = bgconfig[currentStage].states[bgstate_name]
    if not voidp(bgstate) then
      num = bgconfig[currentStage].states.findPos(bgstate_name)
      glob.managers.savegame.unlockBG(currentStage, num)
      bgnode = node["bg"]
      ks = Keys(bgstate)
      repeat with nid in ks
        n = bgnode[nid]
        if not voidp(n) then
          vis = bgstate[nid]
          n.setVisible(vis)
          if not voidp(old_bgstate) then
            old_vis = old_bgstate[nid]
            if old_vis and not vis then
              disappearNodes.add(n)
            else
              if vis and not old_vis then
                appearNodes.add(n)
              end if
            end if
          end if
          next repeat
        end if
        nothing()
      end repeat
    end if
  end if
  b = lvl[#bar]
  if not voidp(b) then
    if not voidp(bar) then
      bar.destroy()
      bar = VOID
    end if
    pos = b.chairs[1].pos
    bar = script("bar_manager").new([#room_mgr: me, #chairs: b.chairs])
    if not grid.placeObject(bar, pos) then
      complain("couldn't place bar")
    else
      dt = []
      px = pos[1] + 4
      py = pos[2]
      repeat with i = 0 to 7
        dt.add(grid.getTileAt(px, py + i))
      end repeat
      bar.init(dt)
      if myItems.getOne(#bar) = 0 then
        appearNodes.add(bar.node)
      end if
      previousItems.add(#bar)
      bar.setRestaurant(currentStage)
    end if
  end if
  repeat with tdata in lvl.tables
    pos = tdata.chairs[1].pos
    t = script("table", "scripts.game").new([#room_manager: me, #capacity: tdata.chairs.count, #parentNode: tableLayer, #tablenum: tables.count + 1, #data: tdata, #stage: currentStage])
    if grid.placeObject(t, pos - [0, 1]) then
      tables.add(t)
      destTile = grid.getTileAt(pos[1] + 2, pos[2] - 2)
      t.setDestTile(destTile)
      if myTables.getOne(tdata.tiles) = 0 then
        appearNodes.add(t.views[1].node)
      end if
      previousTables.add(tdata.tiles)
      next repeat
    end if
    complain("couldn't place table")
  end repeat
  repeat with i = 1 to foodnodes.count
    fn = foodnodes[i]
    fn_loc = fn.getLoc()
    t = grid.getTileNearest(fn_loc)
    if (t.occupant.type <> #empty) and (t.occupant.type <> #Full) then
      next repeat
    end if
    obj = script("foodspot").new([#node: fn, #room_mgr: me, #timeMultiplier: timeMultiplier])
    if not grid.placeObject(obj, t.pos - [1, 0]) then
      complain("couldn't place foodspot")
      obj.destroy()
      next repeat
    end if
    if obj.Tile <> t then
      nothing()
    end if
    obj.setDestTile(grid.getTileAt(t.pos[1], t.pos[2] + 1))
    kitchen.addSpot(obj)
  end repeat
  if game_mgr.playMode <> #endless then
    if lvl.drinkspot.count then
      n = stationNodes[#drinkspot]
      t = grid.getTileNearest(n.getLoc())
      if (t.occupant.type = #empty) or (t.occupant.type = #Full) then
        obj = script("drinkspot").new([#node: n, #room_mgr: me, #timeMultiplier: timeMultiplier])
        if not grid.placeObject(obj, t.pos) then
          complain("couldn't place drinks")
          stationNodes["drinkspot"].setVisible(0)
        else
          kitchen.addSpot(obj)
          if myItems.getOne(#drinks) = 0 then
            appearNodes.add(stationNodes["drinkspot"])
          end if
          previousItems.add(#drinks)
        end if
        obj.setDestTile(grid.getTileAt(t.pos[1], t.pos[2] + 2))
        stations[#drinks] = obj
      end if
    else
      stationNodes["drinkspot"].setVisible(0)
    end if
  end if
  repeat with k in [#ticketspot, #busspot]
    n = stationNodes[k]
    if voidp(n) then
      next repeat
    end if
    t = grid.getTileNearest(n.getLoc())
    if (t.occupant.type <> #empty) and (t.occupant.type <> #Full) then
      next repeat
    end if
    obj = script(string(k)).new([#node: n, #room_mgr: me, #timeMultiplier: timeMultiplier])
    obj.setDestTile(grid.getTileAt(t.pos[1], t.pos[2] + (k = #ticketspot)))
    stations[k] = obj
    kitchen.addSpot(obj)
  end repeat
  if not voidp(lvl[#obstacles]) then
    repeat with i in lvl.obstacles
      ob = script("obstacle").new([#parentNode: node])
      if not grid.placeObject(ob, i) then
        complain("couldn't place obstacle")
        ob.destroy()
        next repeat
      end if
      obstacles.add(ob)
    end repeat
  end if
  if not voidp(lvl_all[#podium]) and not voidp(podiumNode) then
    if not voidp(podium) then
      podium.destroy()
      podium = VOID
    end if
    podium = script("podium", "scripts.game").new([#node: podiumNode, #game_mgr: game_mgr, #room_mgr: me])
    t = grid.getTileNearest(podiumNode.getLocRel())
    if not grid.placeObject(podium, t.pos) then
      complain("couldn't place podium")
    else
      podium.setDestTile(grid.getTileAt(t.pos[1], t.pos[2] - 1))
      podium.setConfig(lvl_all.podium)
      podium.setReady(1)
      if myItems.getOne(#podium) = 0 then
        appearNodes.add(podiumNode)
      end if
      previousItems.add(#podium)
    end if
  end if
  pups = lvl_all[#powerups]
  if not voidp(pups) then
    repeat with i in pups
      game_mgr.addInventoryItem(i)
    end repeat
  end if
  game_mgr.setSnacks(lvl_all[#snacks], lvl_all[#snackPercent])
  waiter.reset()
  waiter.arriveAtTile(stations.ticketspot.getTile())
  call(#hideError, waiter.views)
  call(#setDirection, waiter.views, "down")
  grid.updateLocZ()
end
