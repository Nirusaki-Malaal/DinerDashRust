property callbacks, transitionsTo, transitionsFrom

on new me
  callbacks = [:]
  transitionsTo = [:]
  transitionsFrom = [:]
  return me
end

on destroy me
  callbacks.deleteAll()
  transitionsTo.deleteAll()
  transitionsFrom.deleteAll()
end

on setCallback me, sym, cb
  callbacks[sym] = cb
end

on setTransitionFrom me, st, cb
  transitionsFrom.setProp(st, cb)
end

on setTransitionTo me, st, cb
  transitionsTo.setProp(st, cb)
end

on transitionFrom me, st
  cb = transitionsFrom.getProp(st)
  if voidp(cb) then
    return 
  end if
  call(cb.handler, cb.target, cb[#Args])
end

on enterState me
  cb = callbacks[#enterState]
  if voidp(cb) then
    return 
  end if
  call(cb.handler, cb.target, cb[#Args])
end

on leaveState me
  cb = callbacks[#leaveState]
  if voidp(cb) then
    return 
  end if
  call(cb.handler, cb.target, cb[#Args])
end

on transitionTo me, st
  cb = transitionsTo.getProp(st)
  if voidp(cb) then
    return 
  end if
  call(cb.handler, cb.target, cb[#Args])
end
