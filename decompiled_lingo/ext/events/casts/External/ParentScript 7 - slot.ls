property callback, signals

on new me, cb
  callback = cb
  return me
end

on connectSignal me, sig
  sig.addSlot(me)
  me.addSignal(sig)
end

on disconnectSignal me, sig
  sig.removeSlot(me)
  me.removeSignal(sig)
end

on disconnectAll me
  repeat with i in signals
    i.removeSlot(me)
  end repeat
  signals.deleteAll()
end

on destroy me
  me.disconnectAll()
  callback.deleteAll()
end

on fire me, Args
  call(callback.handler, callback.target, Args, callback[#Args])
end
