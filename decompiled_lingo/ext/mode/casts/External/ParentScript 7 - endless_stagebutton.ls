property button, sparkle, node, animArgs

on new me, Args
  button = newButton(Args)
  node = Args.node
  sparkle = Args.sparkle.children[1]
  sparkle.setVisible(0)
  sparkle.animator.setGroup("sparkle")
  Args.node.addMouseListener(me)
  animArgs = [#loopCount: 0, #callback: [#target: me, #frameMS: 100, #handler: #animEnd]]
  return me
end

on mouseEnter me
  sparkle.setVisible(1)
  sparkle.setAnimation("sparkle")
end

on mouseLeave me
  sparkle.animator.stop()
  sparkle.setVisible(0)
end

on destroy me
  button.destroy()
  button = VOID
  node.removeMouseListener(me)
  node = VOID
  sparkle = VOID
end

on setCallback me, cb
  button.setCallback(cb)
end

on setDisabled me, d
  button.setDisabled(d)
  sparkle.setVisible(0)
end

on setVisible me, v
  button.setVisible(v)
  sparkle.setVisible(0)
end
