property node, timer

on new me, Args
  node = Args.node
  node.setVisible(1)
  me.startAnim()
  return me
end

on destroy me
  if not voidp(timer) then
    timer.destroy()
  end if
  if not voidp(node) then
    node.setVisible(0)
  end if
  voidRefs(me)
end

on startAnim me
  if voidp(node) then
    return 
  end if
  node.setAnimation("start", [#loopCount: 1, #frameMS: 75, #animProps: [#member, #loc]])
  node.queueAnimation("loop", [#loopCount: 4 + random(10), #frameMS: 75, #animProps: [#member, #loc]])
  node.queueAnimation("end", [#loopCount: 1, #frameMS: 75, #animProps: [#member, #loc], #callback: [#target: me, #handler: #endAnim]])
end

on endAnim me
  if not voidp(timer) then
    timer.destroy()
  end if
  if voidp(node) then
    return 
  end if
  timer = timedCall(1000 + random(2000), [#target: me, #handler: #startAnim])
end
