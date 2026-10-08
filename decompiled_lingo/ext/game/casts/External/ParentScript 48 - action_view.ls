property node, text, num, num_bg, names

on new me, n
  node = n
  text = n["action_txt"]
  num = n["tablenum"]
  num_bg = n["tablenum_bg"]
  node.setVisible(0)
  names = [:]
  names[#drinkspot] = "GET DRINKS"
  names[#foodspot] = "PICK UP FOOD"
  names[#table] = "GO TO TABLE"
  names[#ticketspot] = "TICKET"
  names[#busspot] = "BUS DISHES"
  names[#check] = "GIVE CHECK"
  return me
end

on setVisible me, v
  node.setVisible(v)
end

on destroy me
  node = VOID
  num = VOID
  num_bg = VOID
end

on showDest me, d, arg
  if voidp(d) then
    node.setVisible(0)
    return 
  end if
  node.setVisible(1)
  if not voidp(arg) then
    typ = arg
  else
    typ = d.type
  end if
  text.setText(names[typ])
  tbl = VOID
  if typ = #foodspot then
    tbl = d.food
  else
    if (typ = #table) or (typ = #check) then
      tbl = d
    end if
  end if
  if not voidp(tbl) then
    num.setText(string(tbl.tablenum))
    num.setVisible(1)
    num_bg.setVisible(1)
  else
    num.setVisible(0)
    num_bg.setVisible(0)
  end if
end
