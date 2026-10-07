property node, startVal, endVal, diff, lengthMS, currentMS, getter, setter, eventGroup, callback

on new me, Args
  if not voidp(Args) then
    me.startTween(Args)
  end if
  return me
end

on destroy me
  unsubscribe(me, #exitFrame, eventGroup)
  callback = VOID
  node = VOID
end

on startTween me, Args
  lengthMS = Args.ms
  node = Args.node
  setter = Args.setter
  if node.handlers().getOne(setter) = 0 then
    return 
  end if
  startVal = Args.start
  endVal = Args.end
  eventGroup = Args[#group]
  callback = Args[#callback]
  if not voidp(callback) then
    if callback.target.handlers().getOne(callback.handler) = 0 then
      callback = VOID
    end if
  end if
  diff = endVal - startVal
  currentMS = 0
  if lengthMS = 0 then
    me.endTween()
  else
    call(setter, node, startVal)
    subscribe(me, #exitFrame, eventGroup)
  end if
end

on exitFrame me, ms
  if voidp(node) then
    unsubscribe(me, #exitFrame, eventGroup)
    return 
  end if
  currentMS = min(lengthMS, currentMS + ms)
  if currentMS = lengthMS then
    me.endTween()
    return 
  end if
  frac = float(currentMS) / lengthMS
  val = (frac * diff) + startVal
  call(setter, node, val)
end

on endTween me
  unsubscribe(me, #exitFrame, eventGroup)
  if voidp(node) then
    return 
  end if
  call(setter, node, endVal)
  if not voidp(callback) then
    call(callback.handler, callback.target, callback[#Args])
  end if
  node = VOID
end
