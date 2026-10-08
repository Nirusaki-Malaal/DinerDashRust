global glob

on new me, nodes, bgnode, bar
  cn = nodes.stool
  sn = nodes.sitter
  cnum = nodes.stoolnum
  ln = nodes.legs
  s = nodes.setting
  t = nodes.mousetrap
  dirty = nodes.dirty
  ts = nodes.tablestuff
  tnode = ts[ts.count]
  tstuff = tnode.newNode(getNodeData("bar_tablestuff"))
  numloc = (tnode.getLoc() - ts[1].getLoc()) / 2.0
  hr = tstuff["heartrow"]
  hloc = hr.getLocRel()
  tstuff.setLoc(tstuff.getLoc() - numloc)
  hr.setLocRel(hloc)
  Args = [#chair_nodes: cn, #sitter_nodes: sn, #settings: s, #chair_nums: cnum, #legnodes: ln, #table_nodes: t, #isBar: 1, #bar: bar, #tablestuff: tstuff, #dirty_nodes: dirty, #node: bgnode]
  return script("table_view").new(Args)
end
