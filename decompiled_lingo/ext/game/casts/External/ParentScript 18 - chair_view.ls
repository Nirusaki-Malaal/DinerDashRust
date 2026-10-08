property view_mgr, node, numnode, flagmems, tableview, dragTarget, dragGroup, dragData, colormems, chairobj, bonusview, particle, position, mouserect, castname, chname
global glob

on new me, Args
  view_mgr = glob.managers.view
  node = Args.chair
  if not voidp(Args[#mouserect]) then
    mouserect = Args.mouserect
  else
    mouserect = node
  end if
  numnode = Args.number
  position = Args.pos
  if position contains "bar" then
    chname = "stool_"
    castname = "gfx.room"
    flip = 1
  else
    chname = "chair_"
    castname = Args[#castname]
    if voidp(castname) then
      castname = "gfx.diner"
    end if
    flip = node.xflip
  end if
  colormems = [:]
  repeat with i in [#none, #red, #green, #blue, #purple, #yellow, #grey]
    colormems[i] = member(chname & i, castname)
  end repeat
  if flip then
    str = "_flip"
  else
    str = EMPTY
  end if
  flagmems = [:]
  repeat with i in [#red, #green, #blue, #purple, #yellow]
    ls = []
    repeat with j = 1 to 4
      ls.add(member(i & "_" & j + 1 & str))
    end repeat
    flagmems[i] = ls
  end repeat
  dragData = [#tableNode: Args.tableNode]
  dragTarget = script("drag_target").new([#node: mouserect, #recursive: 0, #data: dragData])
  dragGroup = getDragGroup("game.tables")
  return me
end

on setRestaurant me, rname
  castname = "gfx." & rname
  repeat with i in [#none, #red, #green, #blue, #purple, #yellow, #grey]
    colormems[i] = member(chname & i, castname)
  end repeat
end

on setTableNode me, n
  tableNode = n
  dragData.tableNode = n
end

on setTable me, t
  dragData[#table] = t
end

on setObject me, obj
  if obj = chairobj then
    return 
  end if
  if not voidp(chairobj) then
    chairobj.removeView(me)
  end if
  chairobj = obj
  if not voidp(chairobj) then
    dragTarget.data[#chair] = chairobj
    dragData[#chair] = obj
    dragData[#table] = obj.table
    dragGroup.addTarget(dragTarget)
    chairobj.addView(me)
  else
    dragGroup.removeTarget(dragTarget)
  end if
  me.update()
end

on removeObject me, obj
  if obj = chairobj then
    me.setObject(VOID)
  end if
end

on showBonus me, pts
  view_mgr.showChairBonus(node, pts)
end

on update me
  if voidp(chairobj) then
    node.setVisible(0)
    return 
  end if
  c = chairobj.color
  cm = colormems[chairobj.color]
  if voidp(cm) then
    cm = colormems.none
  end if
  bonus = chairobj.currentBonus
  if bonus > 1 then
    fm = flagmems[c]
    if not voidp(fm) then
      numnode.setMember(fm[max(1, min(bonus - 1, 4))])
      if voidp(chairobj.sitter) then
        numnode.setVisible(1)
      else
        numnode.setVisible(0)
      end if
    end if
  else
    numnode.setVisible(0)
  end if
  node.setMember(cm)
  node.setVisible(1)
end

on hideFlag me
  numnode.setVisible(0)
end

on destroy me
  if not voidp(chairobj) then
    chairobj.removeView(me)
  end if
  node = VOID
  numnode = VOID
  bonusview = VOID
  chairobj = VOID
  dragTarget.destroy()
  dragTarget = VOID
  dragGroup = VOID
  dragData.deleteAll()
  tableview = VOID
end
