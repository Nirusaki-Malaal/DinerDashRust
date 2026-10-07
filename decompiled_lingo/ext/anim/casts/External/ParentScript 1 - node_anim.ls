property node, animdata, animstate, currentAnim, currentFrame, pingpong, loopCount, loopNum, frameTimer, frameMS, defaultFrameMS, nodeHandlers, animProps, queue, callback, animname, defaultAnimProps

on new me, Args
  node = Args.node
  queue = []
  me.setAnimData(Args.data)
  oneshot = Args[#oneshot]
  pingpong = Args[#pingpong]
  frameMS = Args[#frameMS]
  if voidp(frameMS) then
    frameMS = 100
  end if
  defaultFrameMS = frameMS
  defaultAnimProps = [#member, #loc, #rotation, #regPoint, #xflip, #yflip]
  me.setAnimProps(defaultAnimProps)
  frameTimer = script("timeout").new()
  frameTimer.setGroup(Args[#group])
  frameTimer.setPeriod(frameMS)
  frameTimer.setCallback([#target: me, #handler: #nextFrame])
  return me
end

on setAnimData me, dat, ap
  animdata = dat
  currentData = animdata[1]
  currentFrame = 1
end

on setGroup me, gr
  frameTimer.setGroup(gr)
end

on setAnimProps me, ap
  if animProps = ap then
    return 
  end if
  animProps = ap
  nodeHandlers = [:]
  repeat with i in animProps
    if i = #blend then
      nodeHandlers[i] = #setAlpha
      next repeat
    end if
    nodeHandlers[i] = symbol("set" & i)
  end repeat
end

on setupLocs me
  pt = node.getRegPoint()
  repeat with j in animdata
    repeat with i in j
      loc = i[#loc]
      if voidp(loc) then
        next repeat
      end if
      i[#loc] = loc
    end repeat
  end repeat
end

on setFrameMS me, ms
  frameMS = ms
  frameTimer.setPeriod(frameMS)
end

on setCallback me, cb
  callback = cb
end

on setFrame me, fr
  if voidp(currentAnim) then
    return 
  end if
  fr = max(1, min(currentAnim.count, fr))
  me.updateNode(currentAnim[fr])
end

on setPaused me, b
  frameTimer.setPaused(b)
end

on start me
  if voidp(currentAnim) then
    return 
  end if
  currentFrame = 1
  subscribe(me, #exitFrame)
  me.nextFrame()
  frameTimer.start()
end

on stop me
  unsubscribe(me, #exitFrame)
  frameTimer.stop()
  currentAnim = VOID
  queue.deleteAll()
end

on setAnimation me, nm, Args
  animname = nm
  if Args.ilk = #propList then
    loopCount = Args[#loopCount]
    pingpong = Args[#pingpong]
    if not voidp(Args[#callback]) then
      me.setCallback(Args[#callback])
    end if
    fms = Args[#frameMS]
    if not voidp(Args[#animProps]) then
      me.setAnimProps(Args.animProps)
    else
      me.setAnimProps(defaultAnimProps)
    end if
  else
    me.setCallback(VOID)
  end if
  loopNum = 1
  if voidp(fms) then
    fms = defaultFrameMS
  end if
  frameMS = fms
  frameTimer.stop()
  frameTimer.setPeriod(fms)
  dat = animdata[nm]
  if voidp(dat) then
    return 
  end if
  animstate = nm
  currentAnim = dat
  currentFrame = 1
  me.start()
end

on queueAnimation me, nm, Args
  queue.add([#name: nm, #Args: Args])
  me.updateQueue()
end

on updateQueue me
  if not voidp(currentAnim) then
    return 
  end if
  if queue.count then
    q = queue[1]
    queue.deleteAt(1)
    me.setAnimation(q.name, q.Args)
  else
    me.stop()
    if not voidp(callback) then
      call(callback.handler, callback.target, callback[#Args])
    end if
  end if
end

on nextFrame me
  if voidp(currentAnim) then
    return 
  end if
  if currentFrame > currentAnim.count then
    if loopCount then
      loopNum = loopNum + 1
      if loopNum > loopCount then
        currentAnim = VOID
        me.updateQueue()
        return 
      else
        currentFrame = max(1, currentFrame mod currentAnim.count)
      end if
    else
      currentFrame = max(1, currentFrame mod currentAnim.count)
    end if
  end if
  me.updateNode(currentAnim[currentFrame])
  currentFrame = currentFrame + 1
end

on updateNode me, dat
  if voidp(node) then
    return 
  end if
  repeat with i in animProps
    h = nodeHandlers[i]
    v = dat[i]
    call(h, node, v)
  end repeat
end

on destroy me
  me.stop()
  if not voidp(frameTimer) then
    frameTimer.destroy()
    frameTimer = VOID
  end if
  callback = VOID
  node = VOID
end
