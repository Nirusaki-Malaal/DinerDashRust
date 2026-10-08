property glob, room, queue, waiter, levelScore, totalScore, levelMS, currentMS, node, interface_node, waitingGroups, newGroups, arriveTimer, rampTimer, rampMS, leveldata, currentLevel, currentStage, currentDifficulty, stagedata, modedata, subleveldata, playMode, inventory, buttons, moneyjar, moneylevel, customersLeft, customersServed, totalStars, numStars, minimumTip, bonusTip, starmeter, clock, closing, eventText, eventTextHome, eventTextUp, accumulatedUpgrades, upgradeButton, goalText, goalColor, bonusColor, closingTimeMS, closingTimer, save_mgr, config_mgr, timeMultiplier, stateTimeMultipliers, upgrades, maxStars, startingStars, availableSnacks, snackPercent, tutorial, tutorialMode, endlessInterface

on new me, g
  glob = g
  availableSnacks = []
  waitingGroups = []
  newGroups = []
  timeMultiplier = [#starting: 1.0, #current: 1.0, #minimum: 1.0, #decrease: 0.0]
  stateTimeMultipliers = [:]
  repeat with i in [#waiting, #deciding, #ordered, #eating, #check]
    stateTimeMultipliers[i] = 1.0
  end repeat
  return me
end

on init me
  grp = getNodeGroup("game")
  m1 = the milliSeconds
  node = grp.newNode(getNodeData("game"))
  grp.setVisible(0)
  go(the frame)
  put "make game node took " & the milliSeconds - m1 & " ms"
  interface_node = grp.newNode(getNodeData("game_interface"))
  go(the frame)
  interface_node["closingtime"].setVisible(0)
  interface_node.setLoc(point(2, 0))
  interface_node.moveToFront()
  interface_node.setZOffset(900000)
  goalText = interface_node["goal"]
  goalColor = rgb(0, 255, 255)
  bonusColor = rgb(255, 0, 255)
  endlessInterface = interface_node["endless_score"]
  endlessInterface.setVisible(0)
  upgradeButton = newButton([#node: endlessInterface["upgradebutton"], #members: [#rollover: member("upgrade_overstate", "gfx.screens")], #callback: [#target: me, #handler: #clickUpgrade]])
  eventText = [:]
  eventText[#newItem] = endlessInterface["new_item"]
  eventText[#goal] = interface_node["goal_complete"]
  eventText[#bonus] = interface_node["bonus_complete"]
  eventText[#newStar] = interface_node["new_star"]
  eventTextHome = [:]
  repeat with i in [#goal, #bonus, #newStar]
    eventTextHome[i] = eventText[i].getLoc()
  end repeat
  eventTextUp = eventTextHome[1] - point(0, 400)
  call(#setVisible, eventText, 0)
  tutorial = script("tutorial_manager").new()
  tutorialMode = 0
  clock = script("level_clock").new(interface_node["clock_hand"])
  cfg = getConfigData()
  config_mgr = glob.managers.config
  startingStars = cfg.startingStars
  maxStars = cfg.maxStars
  closingTimeMS = cfg.closingTimeMS
  currentLevel = 1
  save_mgr = glob.managers.savegame
  inventory = [:]
  starmeter = script("star_meter").new(interface_node["star_meter"])
  moneyjar = interface_node["moneyjar"]
  moneylevel = 0
  game_cfg = glob.managers.config.game
  upgrades = [:]
  m1 = the milliSeconds
  node["bg"].setZOffset(-1000)
  room = script("room_manager", "scripts.game").new(glob, node)
  put "make room took" && the milliSeconds - m1 && " ms"
  queue = script("waitqueue", "scripts.game").new([#room_manager: room, #game_manager: me, #node: node["waitqueue"]])
  bmems = [#normal: member("button.normal", "gfx.screens"), #pressed: member("button.down", "gfx.screens"), #rollover: member("button.rollover", "gfx.screens")]
  buttons = [:]
  buttons[#quitgame] = newButton([#node: interface_node["quitgame"], #members: [#rollover: member("pauseButton_on", "gfx.screens")], #callback: [#target: glob.managers.mode, #handler: #setMode, #Args: #pause]])
  if cfg[#cheat] then
    buttons[#skipLevel] = newButton([#node: interface_node["skiplevel"], #callback: [#target: me, #handler: #winLevel]])
  else
    interface_node["skiplevel"].setVisible(0)
  end if
  go(the frame)
  nothing()
end

on startGame me
  put "starting game"
  me.setLevelScore(0)
  totalScore = 0
  call(#setVisible, eventText, 0)
  upgrades.deleteAll()
  upgrades.deleteAll()
  room.kitchen.resetSpeed()
  room.waiter.resetSpeed()
  repeat with s in stateTimeMultipliers
    s = 1.0
  end repeat
  room.setTimeMultiplier(1.0)
  if playMode = #endless then
    leveldata = config_mgr.getLevelData([#mode: #endless, #stage: currentStage, #level: currentDifficulty])
    upgradeButton.setVisible(0)
    me.loadLevel()
  else
    room.setTimeMultiplier(1.0)
  end if
  me.startLevel()
end

on removeUpgrades me
  upgrades.deleteAll()
  room.kitchen.resetSpeed()
  room.waiter.resetSpeed()
  repeat with s = 1 to stateTimeMultipliers.count
    stateTimeMultipliers[s] = 1.0
  end repeat
  room.setTimeMultiplier(1.0)
end

on stopGame me
  call(#destroy, waitingGroups)
  waitingGroups.deleteAll()
  call(#destroy, newGroups)
  newGroups.deleteAll()
  room.destroyLevel()
  if not voidp(arriveTimer) then
    arriveTimer.destroy()
    arriveTimer = VOID
  end if
  if not voidp(rampTimer) then
    rampTimer.destroy()
    rampTimer = VOID
  end if
  unsubscribe(me, #exitFrame)
  if tutorialMode then
    tutorial.destroy()
  end if
end

on scheduleRamp me
  if not voidp(rampTimer) then
    rampTimer.destroy()
    rampTimer = VOID
  end if
  rampTimer = timedCall(rampMS, [#target: me, #handler: #incrementLevel])
end

on startLevel me
  me.showTipProgress()
  customersServed = 0
  me.setPaused(0)
  me.scheduleNextArrival(1)
  if playMode = #endless then
    me.scheduleRamp()
  else
    tutorialMode = (currentStage = 1) and (currentLevel = 1)
    if tutorialMode then
      tutorial.start()
    end if
  end if
  subscribe(me, #exitFrame)
end

on skipLevel me
  me.incrementLevel()
  me.startLevel()
end

on newGame me, Args
  if (Args.stage > modedata.numStages) or (Args.stage < 1) then
    return 
  end if
  currentStage = Args.stage
  stagedata = config_mgr.getStageData([#mode: playMode, #stage: currentStage])
  currentLevel = min(stagedata.numLevels, currentLevel)
  totalScore = integer(Args[#score])
  inventory.deleteAll()
  me.removeUpgrades()
  if Args[#mode] = #endless then
    currentLevel = 1
    currentDifficulty = Args.difficulty
    leveldata = config_mgr.getLevelData([#mode: playMode, #stage: currentStage, #level: currentDifficulty])
    upgrades.deleteAll()
    tup = leveldata[#tableUpgrades]
    room.resetObjects()
    if voidp(tup) then
      tup = []
    end if
    upgrades[#table] = tup.duplicate()
    upgrades[#powerups] = config_mgr.powerups.duplicate()
  else
    currentLevel = Args.level
    leveldata = config_mgr.getLevelData([#mode: playMode, #stage: currentStage, #level: currentLevel])
  end if
  totalStars = Args[#totalStars]
  if voidp(totalStars) then
    totalStars = leveldata[#numStars]
  end if
  if voidp(totalStars) then
    totalStars = stagedata[#numStars]
  end if
  if voidp(totalStars) then
    totalStars = startingStars
  end if
  numStars = Args[#numStars]
  if voidp(numStars) then
    numStars = totalStars
  end if
end

on setPlayMode me, sym
  playMode = sym
  if playMode = #career then
    modedata = config_mgr.game.career
    interface_node["clockface"].setVisible(1)
    interface_node["clock_hand"].setVisible(1)
    interface_node["notepad"].setVisible(1)
    endlessInterface.setVisible(0)
  else
    modedata = config_mgr.game.endless
    interface_node["clockface"].setVisible(0)
    interface_node["clock_hand"].setVisible(0)
    interface_node["notepad"].setVisible(0)
    endlessInterface.setVisible(1)
    tutorialMode = 0
  end if
end

on setPaused me, bool
  glob.managers.event.setGroupPaused(bool, VOID)
end

on scoreTip me, inf
  customersServed = customersServed + 1
  me.setLevelScore(levelScore + inf.total)
end

on addPoints me, num
  oldscore = levelScore
  me.setLevelScore(max(0, levelScore + num))
  me.showTipProgress(oldscore)
  if playMode = #career then
    return 
  end if
  if levelScore > minimumTip then
    me.endSubLevel()
  end if
end

on showTipProgress me, oldscore
  num = moneylevel
  if levelScore = 0 then
    moneylevel = 0
    moneyjar.setAnimation("full_0")
    return 
  end if
  if levelScore < minimumTip then
    if oldscore >= minimumTip then
      if playMode = #endless then
        accumulatedUpgrades = max(0, accumulatedUpgrades - 1)
        upgradeButton.setVisible(accumulatedUpgrades > 0)
      end if
    end if
    goalText.setColor(goalColor)
    goalText.setText("GOAL: " & commafy(string(minimumTip)))
    inc = integer(float(minimumTip) * 0.20000000000000001)
    moneylevel = max(1, min((levelScore / inc) + 1, 4))
  else
    if (levelScore >= minimumTip) and (playMode = #career) then
      if levelScore >= bonusTip then
        if oldscore < bonusTip then
          eventText.bonus.setLoc(eventTextHome.bonus)
          cb = [#target: me, #handler: #moveBonusUp, #Args: eventText.bonus]
          flashNode([#node: eventText.bonus, #loopCount: 4, #ms: 150, #callback: cb])
        end if
        goalText.setColor(bonusColor)
        goalText.setText("EXPERT: " & commafy(string(bonusTip)))
      else
        if oldscore < minimumTip then
          eventText.goal.setLoc(eventTextHome.goal)
          cb = [#target: me, #handler: #moveBonusUp, #Args: eventText.goal]
          flashNode([#node: eventText.goal, #loopCount: 4, #ms: 150, #callback: cb])
        end if
        goalText.setColor(bonusColor)
        goalText.setText("EXPERT: " & commafy(string(bonusTip)))
      end if
      inc = integer(float(bonusTip) * 0.25)
      moneylevel = max(5, min((levelScore / inc) + 4, 9))
    end if
  end if
  moneyjar.setAnimation("full_" & moneylevel)
  if not voidp(oldscore) and (levelScore > oldscore) then
    moneyjar["dollarsign"].setAnimation("blink", [#loopCount: 5, #frameMS: 250, #animProps: [#blend]])
  end if
end

on moveBonusUp me, n
  moveNodeTo([#node: n, #pt: eventTextUp, #ms: 750, #callback: [#target: n, #handler: #setVisible, #Args: 0]])
end

on setLevelScore me, num
  levelScore = num
  if playMode = #endless then
    totalScore = levelScore
  end if
  interface_node["score"].setText(commafy(string(levelScore)))
end

on destroy me
  unsubscribe(me, #exitFrame)
  if not voidp(arriveTimer) then
    arriveTimer.destroy()
    arriveTimer = VOID
  end if
  if not voidp(room) then
    room.destroy()
    room = VOID
  end if
  if not voidp(starmeter) then
    starmeter.destroy()
    starmeter = VOID
  end if
  if not voidp(clock) then
    clock.destroy()
    clock = VOID
  end if
  if not voidp(waiter) then
    waiter.destroy()
    waiter = VOID
  end if
  if not voidp(queue) then
    queue.destroy()
    queue = VOID
  end if
  call(#destroy, waitingGroups)
  waitingGroups.deleteAll()
  call(#destroy, newGroups)
  newGroups.deleteAll()
  call(#destroy, buttons)
  buttons.deleteAll()
  node.destroy()
  node = VOID
  glob = VOID
end

on checkLevelDone me
  if playMode = #endless then
    return 
  end if
  if (waitingGroups.count = 0) and (newGroups.count = 0) and (room.sitterGroups.count = 0) and not tutorialMode then
    me.closingTime()
  end if
end

on checkClosing me
  if (playMode = #endless) or not closing then
    return 
  end if
  repeat with i in room.tables
    if i.dirty then
      return 
    end if
  end repeat
  if not voidp(closingTimer) then
    closingTimer.destroy()
    closingTimer = VOID
  end if
  closingTimer = timedCall(1250, [#target: me, #handler: #endlevel])
end

on closingTime me
  if closing or (playMode = #endless) then
    return 
  end if
  closing = 1
  delayms = 1250
  repeat with i in room.tables
    if i.dirty then
      interface_node["closingtime"].setVisible(1)
      delayms = closingTimeMS
      exit repeat
    end if
  end repeat
  if not voidp(closingTimer) then
    closingTimer.destroy()
    closingTimer = VOID
  end if
  closingTimer = timedCall(delayms, [#target: me, #handler: #endlevel])
  timedCall(1000, [#handler: #setVisible, #target: interface_node["closingtime"], 3: 0])
end

on winLevel me
  m = glob.managers.mode.currentMode.name
  if m <> #play then
    return 
  end if
  closing = 1
  me.setLevelScore(max(minimumTip, bonusTip))
  if playMode = #endless then
    me.endSubLevel()
  else
    me.endlevel()
  end if
end

on loseLevel me
  closing = 1
  me.setLevelScore(0)
  me.endlevel()
end

on loseGame me
  closing = 1
  numStars = 0
  me.endlevel()
end

on endlevel me
  call(#setVisible, eventText, 0)
  if playMode = #endless then
    hs = save_mgr.submitScore(levelScore, #endless, currentDifficulty)
    dat = [#text: "GAME OVER", #success: 0, #endgame: 1, #score: [#actual: levelScore], #hiscore: hs]
  else
    if not closing then
      return 
    end if
    closing = 0
    interface_node["closingtime"].setVisible(0)
    if tutorialMode then
      tutorial.destroy()
    end if
    success = levelScore >= minimumTip
    dat = [#customers: [#left: customersLeft, #served: customersServed], #score: [#actual: levelScore, #required: minimumTip], #success: success, #stage: currentStage, #level: currentLevel]
  end if
  if success then
    if playMode = #career then
      if levelScore >= bonusTip then
        save_mgr.starLevel([#stage: currentStage, #level: currentLevel])
      end if
      totalScore = totalScore + levelScore
      if levelScore >= bonusTip then
        dat[#expertScore] = 1
      end if
    end if
    dat[#text] = leveldata.text.succeed
    dat[#doodle] = leveldata[#doodle]
    sg = save_mgr.savedGame
    unlock = (sg.stage < currentStage) or ((sg.stage = currentStage) and (sg.level <= currentLevel))
    dat[#lastLevel] = me.incrementLevel()
    if dat[#lastLevel] then
      dat[#hiscore] = save_mgr.submitScore(totalScore, #career)
    end if
    if currentLevel = 1 then
      dat[#endStage] = 1
    end if
  else
    if playMode = #career then
      dat[#text] = leveldata.text.fail
      numStars = numStars - 1
      if numStars < 1 then
        dat[#endgame] = 1
        dat[#hiscore] = save_mgr.submitScore(totalScore, #career)
        starmeter.setFull(min(maxStars, numStars))
      end if
    end if
  end if
  setMode(#endlevel, dat)
  SndMusicEnd()
end

on incrementLevel me
  if playMode = #career then
    if currentLevel < stagedata.numLevels then
      currentLevel = currentLevel + 1
      leveldata = config_mgr.getLevelData([#mode: playMode, #stage: currentStage, #level: currentLevel])
    else
      if currentStage < 5 then
        currentLevel = 1
        currentStage = currentStage + 1
        leveldata = config_mgr.getLevelData([#mode: playMode, #stage: currentStage, #level: currentLevel])
        stagedata = config_mgr.getStageData([#mode: playMode, #stage: currentStage])
        numStars = stagedata.numStars
      else
        return 1
      end if
    end if
    save_mgr.savegame([#stage: currentStage, #level: currentLevel, #score: totalScore, #totalStars: totalStars, #numStars: numStars])
  else
    timeMultiplier.current = max(timeMultiplier.minimum, timeMultiplier.current - timeMultiplier.decrease)
    room.setTimeMultiplier(timeMultiplier.current)
    call(#setTimeMultiplier, waitingGroups, timeMultiplier.current)
    me.scheduleRamp()
  end if
end

on endSubLevel me
  accumulatedUpgrades = accumulatedUpgrades + 1
  if accumulatedUpgrades = 1 then
    upgradeButton.setVisible(1)
  end if
  currentLevel = currentLevel + 1
  if currentLevel > leveldata.numSublevels then
    subleveldata = config_mgr.getSublevelData([#stage: currentStage, #difficulty: currentDifficulty, #sublevel: leveldata.numSublevels])
    last = config_mgr.getSublevelData([#stage: currentStage, #difficulty: currentDifficulty, #sublevel: leveldata.numSublevels - 1])
    subleveldata.goal = subleveldata.goal + ((currentLevel - leveldata.numSublevels) * (subleveldata.goal - last.goal))
    nothing()
  else
    subleveldata = config_mgr.getSublevelData([#stage: currentStage, #difficulty: currentDifficulty, #sublevel: currentLevel])
  end if
  minimumTip = subleveldata.goal
  me.showTipProgress()
  goalText.setColor(goalColor)
  goalText.setText("GOAL: " & commafy(string(minimumTip)))
  endlessInterface["level_txt"].setText(string(currentLevel))
  endlessInterface["heart"].setAnimation("flash", [#loopCount: 1])
end

on getUpgradeSet me
  ugs = []
  has_tbl = 0
  if accumulatedUpgrades = 0 then
    return ugs
  end if
  if upgrades.table.count then
    ugs.add([#type: #table, #level: currentLevel, #name: "new table", #description: "add a new table to the restaurant", #properties: upgrades.table[1]])
    has_tbl = 1
  end if
  poss_ugs = []
  repeat with i in Keys(upgrades.powerups)
    pup = upgrades.powerups[i].levels
    if pup.count then
      poss_ugs.add(i)
    end if
  end repeat
  repeat with i = 1 to 2 + (has_tbl = 0)
    if poss_ugs.count = 0 then
      exit repeat
    end if
    r = poss_ugs[random(poss_ugs.count)]
    lvls = upgrades.powerups[r].levels
    pup = lvls[1]
    ugs.add(pup)
    poss_ugs.deleteOne(r)
  end repeat
  accumulatedUpgrades = accumulatedUpgrades - 1
  return ugs
end

on clickUpgrade me
  setMode(#endless_shop, [#inventory: inventory, #upgrades: accumulatedUpgrades])
end

on addInventoryItem me, it
  t = it.type
  inventory[t] = it
  props = it.properties
  if t = #table then
    upgrades.table.deleteOne(props)
  else
    upgrades.powerups[t].levels.deleteOne(it)
  end if
  case t of
    #table:
      room.addTable(props)
    #drinks:
      room.setDrinkProps(props)
    #podium:
      room.setPodiumProps(props)
    #stereo:
      num = props.patienceMultiplier
      stateTimeMultipliers[#ordered] = num
      stateTimeMultipliers[#check] = num
      stateTimeMultipliers[#snack] = num
      stateTimeMultipliers[#ready] = num
      call(#setStateTimeMultipliers, room.sitterGroups, stateTimeMultipliers)
      call(#setStateTimeMultipliers, waitingGroups, stateTimeMultipliers)
      call(#setStateTimeMultipliers, newGroups, stateTimeMultipliers)
    #maitred:
      call(#setStateTimeMultipliers, waitingGroups, stateTimeMultipliers)
      call(#setStateTimeMultipliers, newGroups, stateTimeMultipliers)
      room.setMimeProps(props)
    #oven:
      if voidp(props[#foodReadyMS]) then
        complain("oven config must contain an attribute 'speed'")
        return 
      end if
      room.kitchen.foodReadyMS = props.foodReadyMS
    #shoes:
      if voidp(props[#speed]) then
        complain("oven config must contain an attribute 'speed'")
        return 
      end if
      room.waiter.setSpeed(props.speed)
  end case
end

on setSnacks me, ls, pc
  availableSnacks = ls
  snackPercent = integer(pc)
  call(#setSnacks, newGroups, ls, pc)
  call(#setSnacks, waitingGroups, ls, pc)
  room.setSnacks(ls, pc)
end

on angryCustomer me, penalty
  customersLeft = customersLeft + 1
  if playMode = #endless then
    numStars = numStars - 1
    starmeter.setFull(min(maxStars, numStars))
    if numStars = 0 then
      me.endlevel()
    end if
  end if
  me.addPoints(-penalty)
end

on happyCustomer me
  customersServed = customersServed + 1
end

on removeWaitingGroup me, gr
  waitingGroups.deleteOne(gr)
  queue.update()
  me.checkLevelDone()
  me.updateWaitingGroups()
end

on updateWaitingGroups me
  repeat with i = 1 to min(queue.size, waitingGroups.count)
    w = waitingGroups[i]
    if w.getState() <> #waiting then
      w.setState(#waiting)
    end if
  end repeat
end

on exitFrame me
  if playMode = #endless then
    return 
  end if
  if voidp(arriveTimer) then
    return 
  end if
  clock.setMark(currentMS + arriveTimer.getMS())
end

on addWaitingGroup me, gr
  if waitingGroups.getOne(gr) then
    return 
  end if
  if playMode = #career then
    currentMS = currentMS + gr.delay
    clock.setMark(currentMS)
  end if
  waitingGroups.add(gr)
  me.updateWaitingGroups()
  queue.update()
  me.scheduleNextArrival()
  SndSFX("sfx_party_arrive_1")
end

on scheduleNextArrival me, now
  if playMode = #endless then
    newGroups.add(me.getRandomGroup())
  else
    if newGroups.count = 0 then
      if not voidp(arriveTimer) then
        arriveTimer.destroy()
        arriveTimer = VOID
      end if
      room.closingTime()
      return 
    end if
  end if
  g = newGroups[1]
  newGroups.deleteAt(1)
  if now then
    currentMS = currentMS + g.delay
    if playMode = #endless then
      clock.setMark(currentMS)
    end if
    delayms = 0
  else
    delayms = g.delay
  end if
  arriveTimer = timedCall(delayms, [#target: me, #handler: #addWaitingGroup, #Args: g])
end

on loadLevel me
  room.destroyLevel()
  room.loadStage(stagedata.id)
  currentLevel = max(1, min(currentLevel, stagedata.numLevels))
  call(#destroy, waitingGroups)
  waitingGroups.deleteAll()
  if not voidp(arriveTimer) then
    arriveTimer.destroy()
    arriveTimer = VOID
  end if
  queue.update()
  customersLeft = 0
  customersServed = 0
  interface_node["level_txt"].setText(string(currentLevel))
  newGroups.deleteAll()
  waitingGroups.deleteAll()
  queue.update()
  lvl = leveldata
  me.setLevelScore(0)
  if playMode = #career then
    interface_node["level_txt"].setText(string(currentLevel))
    grps = lvl.groups
    minimumTip = lvl.minimumTip
    bonusTip = lvl[#bonusGoal]
    if voidp(bonusTip) then
      bonusTip = 2 * minimumTip
    end if
    goalText.setColor(goalColor)
    goalText.setText("GOAL: " & commafy(string(minimumTip)))
    levelMS = 0
    currentMS = 0
    repeat with i in grps
      cfg = getGroupConfig(i.type)
      ms = i.queueTimeMS
      levelMS = levelMS + ms
      newGroups.add(script("sitterGroup").new([#sitters: i[#customers], #room_manager: room, #game_manager: me, #config: cfg, #delay: i.queueTimeMS, #type: i.type]))
    end repeat
    clock.setEndMark(levelMS)
    clock.setMark(0)
  else
    endlessInterface["level_txt"].setText("1")
    subleveldata = config_mgr.getSublevelData([#stage: currentStage, #difficulty: currentDifficulty, #sublevel: min(leveldata.numSublevels, currentLevel)])
    timeMultiplier.starting = lvl.startingTimeMultiplier
    timeMultiplier.current = lvl.startingTimeMultiplier
    timeMultiplier.minimum = lvl.timeMultiplierMinimum
    timeMultiplier.decrease = lvl.timeMultiplierDecrease
    room.setTimeMultiplier(timeMultiplier)
    rampMS = lvl.difficultyIncreaseMS
    accumulatedUpgrades = 0
    upgradeButton.setVisible(0)
    minimumTip = subleveldata.goal
    goalText.setColor(goalColor)
    goalText.setText("GOAL: " & commafy(string(minimumTip)))
    clock.setEndMark(100)
    clock.setMark(0)
    newGroups.add(me.getRandomGroup())
    numStars = totalStars
  end if
  starmeter.setCapacity(min(maxStars, totalStars))
  starmeter.setFull(min(maxStars, numStars))
  room.loadLevel(lvl)
  if playMode = #endless then
    room.updateBGState()
  end if
  me.setPaused(1)
  SndMusicStart("music_game")
end

on getStartlevelText me
  if playMode = #career then
    lvl = leveldata
    return [#text: lvl.text.intro]
  else
    return [#text: "hit button to begin"]
  end if
end

on loadStage me, stname
end

on testNewStar me
  me.addStar()
  me.addPoints(bonusTip)
end

on addStar me
  totalStars = min(maxStars, totalStars + 1)
  numStars = min(maxStars, totalStars + 1)
  starmeter.setCapacity(totalStars)
  starmeter.setFull(numStars)
  eventText.newStar.setLoc(eventTextHome.newStar)
  cb = [#target: me, #handler: #moveBonusUp, #Args: eventText.newStar]
  flashNode([#node: eventText.newStar, #loopCount: 4, #ms: 150, #callback: cb])
end

on setStateTimeMultipliers me, ls
  if voidp(ls) then
    ls = [:]
  end if
  repeat with i in Keys(stateTimeMultipliers)
    v = ls[i]
    if voidp(v) then
      stateTimeMultipliers[i] = 1.0
      next repeat
    end if
    stateTimeMultipliers[i] = v
  end repeat
end

on getRandomGroup me
  if playMode <> #endless then
    complain("game_manager.getRandomGroup() is only for endless mode")
    return 
  end if
  subleveldata = config_mgr.getSublevelData([#stage: currentStage, #difficulty: currentDifficulty, #sublevel: min(leveldata.numSublevels, currentLevel)])
  grps = subleveldata.groups
  cols = [#green, #red, #blue, #yellow, #purple]
  if voidp(subleveldata[#randomColors]) then
    randomColors = cols
  else
    randomColors = []
    repeat with i in subleveldata.randomColors
      if cols.getOne(i) then
        randomColors.add(i)
      end if
    end repeat
  end if
  r = random(subleveldata.totalGroupFrequency)
  tot = 0
  repeat with g in grps
    tot = tot + g.Frequency
    if r < tot then
      exit repeat
    end if
  end repeat
  cfg = getGroupConfig(g.type)
  if voidp(g.type) then
    complain("no group data for type " & g.type)
    return 
  end if
  num = 2
  if random(2) - 1 then
    num = num + random(2)
  end if
  sitters = []
  repeat with i = 1 to num
    if cfg.graphicType = "critic" then
      clr = #none
    else
      clr = randomColors[random(randomColors.count)]
    end if
    sitters.add([#color: clr])
  end repeat
  sd = cfg.stateData
  repeat with i in Keys(stateTimeMultipliers)
    sd[i] = sd[i] * stateTimeMultipliers[i]
  end repeat
  groupobj = script("sitterGroup").new([#room_manager: room, #game_manager: me, #config: cfg, #delay: g.queueTimeMS, #type: g.type, #sitters: sitters])
  groupobj.setTimeMultiplier(timeMultiplier.current)
  return groupobj
end

on activatePodium me, num
  call(#changeMood, waitingGroups, num)
  queue.activatePodium()
end
