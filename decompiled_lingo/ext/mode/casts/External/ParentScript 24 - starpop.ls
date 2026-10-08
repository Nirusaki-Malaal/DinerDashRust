property node, scale, poptime, ms, startscale, tween

on new me, n
  node = n
  startscale = n.getScale()
  scale = 1.19999999999999996
  poptime = 150
  me.grow()
end

on grow me
  node.setScale(startscale * scale)
  state = #expanding
  tween = script("node_tween").new()
  cb = [#target: me, #handler: #shrink]
  tween.startTween([#node: node, #setter: #setScale, #group: "titles", #start: startscale, #end: startscale * scale, #ms: poptime, #callback: cb])
end

on shrink me
  if not voidp(tween) then
    tween.destroy()
  end if
  tween = script("node_tween").new()
  cb = [#target: me, #handler: #destroy]
  tween.startTween([#node: node, #setter: #setScale, #group: "titles", #start: startscale * scale, #end: startscale, #ms: poptime, #callback: cb])
end

on destroy me
  if not voidp(tween) then
    tween.destroy()
  end if
  voidRefs(me)
end
