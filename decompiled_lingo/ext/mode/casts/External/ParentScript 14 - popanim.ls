property node, callback, poptime, popsize, poploc, nodescale, home, ms, timergroup, defaults

on new me
  defaults = [:]
  defaults[#poptime] = 200.0
  defaults[#popsize] = 2.0
  defaults[#poploc] = point(15, 10)
  return me
end

on destroy me
  unsubscribe(me, #exitFrame, timergroup)
  node = VOID
  callback = VOID
end

on pop me, Args
  if not voidp(node) then
    me.stop()
  end if
  node = Args.node
  SndSFX("sfx_menu_down_2")
  home = node.getLoc()
  nodescale = node.getScale()
  callback = Args[#callback]
  timergroup = Args[#timergroup]
  repeat with i in [#poptime, #popsize, #poploc]
    v = Args[i]
    if voidp(v) then
      v = defaults[i]
    end if
    me[i] = v
  end repeat
  poptime = float(poptime)
  ms = poptime
  subscribe(me, #exitFrame, timergroup)
  node.setVisible(1)
  me.exitFrame(0)
end

on exitFrame me, frameMS
  if node.voidp then
    return 
  end if
  ms = ms - frameMS
  if ms <= 0 then
    me.stop()
    return 
  end if
  ratio = ms / poptime
  loc = home + (poploc * ratio)
  size = 1.0 + (popsize * ratio)
  node.setScale(size * nodescale)
  node.setLoc(loc)
end

on stop me
  if not voidp(callback) then
    call(callback.handler, callback.target, callback[#Args])
  end if
  unsubscribe(me, #exitFrame, timergroup)
  node.setLoc(home)
  node.setScale(nodescale)
  node = VOID
end
