property node, endMark, currentMark, last_num, animProps, fork

on new me, n
  node = n
  fork = node.children[1]
  endMark = 0.0001
  animProps = [#member, #rotation, #xflip, #yflip]
  return me
end

on setEndMark me, num
  node.setRotation(0)
  node.setScale([1, 1])
  endMark = max(0.0001, num)
end

on setMark me, num
  myint = max(1, min(12, integer(float(num) / endMark * 12)))
  if myint = last_num then
    return 
  end if
  last_num = myint
  fork.setAnimation(string(myint), [#loopCount: 1, #animProps: animProps])
  fork.pSprite.stretch = 0
end

on destroy me
  repeat with i = 1 to me.count
    me[i] = VOID
  end repeat
end
