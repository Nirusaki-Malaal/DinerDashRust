property node, button, mgr
global glob

on new me, m
  mgr = m
  node = mgr.node.newNode(getNodeData("endlevel"))
  button = newButton([#node: node["okbutton"], #callback: [#target: glob.managers.mode, #handler: #setMode, #Args: #play]])
  node.setVisible(0)
  node.setLoc(point(800 - (node.getWidth() / 2), 600 - (node.getHeight() / 2)))
  node.setZOffset(1000000)
  node.moveToFront()
  return me
end

on setVisible me, v
  node.setVisible(v)
end

on destroy me
  node.destroy()
  node = VOID
  button.destroy()
  button = VOID
  mgr = VOID
end
