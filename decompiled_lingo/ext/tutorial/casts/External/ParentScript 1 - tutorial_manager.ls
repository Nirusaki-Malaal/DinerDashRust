property grp, node, arrow, mousetraps, steps, stepNum, currentStep, room_mgr, game_mgr, running, timer, stepnames, initialized
global glob

on init me
  room_mgr = glob.managers.game.room
  game_mgr = glob.managers.game
  steps = []
  repeat with i = 1 to 24
    s = script("tutorial_" & i, "scripts.tutorial").new()
    voidRefs(s)
    steps.add(s)
  end repeat
  grp = getNodeGroup("game")
  node = grp.newNode(getNodeData("tutorial"))
  node.moveToFront()
  mousetraps = []
  repeat with i = 1 to 3
    n = node["mousehole_" & i]
    if not voidp(n) then
      mousetraps.add(n)
      n.setAlpha(0)
    end if
  end repeat
  timer = script("timeout").new()
  timer.setPeriod(500)
  timer.setCallback([#target: me, #handler: #checkStepDone])
  node.setZOffset(999999)
  initialized = 1
end

on start me
  if not initialized then
    me.init()
  end if
  call(#addMouseListener, mousetraps, me)
  stepNum = 1
  me.startStep()
end

on mouseUp me
  sendEvent(#mouseRelease)
end

on startStep me
  if (stepNum < 1) or (stepNum > 24) then
    return 
  end if
  currentStep = steps[stepNum]
  if not currentStep.startStep() then
    return 
  end if
  node.setVisible(1)
  node.setAnimation(string(stepNum), [#loopCount: 1, #animProps: [#member, #loc, #scale, #rotation, #xflip, #yflip]])
  timer.start()
  return 1
end

on checkStepDone me
  if not currentStep.checkDone() then
    return 
  end if
  timer.stop()
  node.setVisible(0)
  Ok = 0
  repeat while not Ok
    stepNum = stepNum + 1
    if stepNum > steps.count then
      me.end()
      return 
    end if
    Ok = me.startStep()
  end repeat
end

on end me
  game_mgr.closingTime()
  me.destroy()
end

on destroy me
  if not voidp(node) then
    node.destroy()
  end if
  if not voidp(timer) then
    timer.destroy()
    timer = VOID
  end if
  if not voidp(steps) then
    call(#destroy, steps)
  end if
  voidRefs(me)
end
