property node, visible, tweener

on new me
  tweener = script("node_tween").new()
  return me
end

on destroy me
  tweener.destroy()
  voidRefs(me)
end

on setNode me, n
  if not voidp(node) then
    node.setAnimation(EMPTY)
  end if
  tweener.endTween()
  node = n
  visible = 0
  if node.voidp then
    return 
  end if
  n.setVisible(0)
  n.setAlpha(0)
end

on setVisible me, v
  if voidp(node) then
    return 
  end if
  if visible = v then
    return 
  end if
  visible = v
  if v then
    node.setVisible(1)
    me.startAnim()
    tweener.startTween([#node: node, #start: node.getAlpha(), #end: 100, #setter: #setAlpha, #ms: 400])
  else
    tweener.startTween([#node: node, #start: node.getAlpha(), #end: 0, #setter: #setAlpha, #ms: 750])
  end if
end

on startAnim me
  if voidp(node) then
    return 
  end if
  node.setAnimation(["chop", "fry"][random(2)], [#loopCount: 1 + random(4), #callback: [#target: me, #handler: #animDone], #animProps: [#loc, #member]])
end

on animDone me
  if voidp(node) then
    return 
  end if
  if not visible then
    return 
  end if
  me.startAnim()
end
