global glob

on strip str
  s = EMPTY
  repeat with i = 1 to str.line.count
    ln = str.line[i]
    if ln contains "--" then
      offs = offset("--", ln) - 1
      if offs > 0 then
        ln = ln.char[1..offset("--", ln) - 1]
      else
        ln = EMPTY
      end if
    end if
    s = s & ln
  end repeat
  return s
end

on devoid x
  xxx = offset("<Void>", x)
  repeat while xxx
    delete x.char[xxx]
    delete x.char[xxx + 4]
    xxx = offset("<Void>", x)
  end repeat
  return x
end

on updateVersionText
  if the runMode = "Author" then
    t = getNetText("version.txt")
    repeat while not netDone(t)
      nothing()
    end repeat
    txt = netTextResult(t)
    member("version_txt", "scripts.config").text = txt
  end if
  return txt
end

on putTextContent
  str = EMPTY
  repeat with i = 1 to the number of castMembers of castLib "leveltext"
    m = member(i, "leveltext")
    if m.type = #text then
      str = str & "---------------" & RETURN & m.name & RETURN & m.text & RETURN
    end if
  end repeat
  str = str & RETURN & "---tutorial---------------"
  repeat with i = 1 to the number of castMembers of castLib "ext_text_temp"
    m = member(i, "ext_text_temp")
    if m.type <> #text then
      next repeat
    end if
    str = str & m.text & RETURN & "---------------------" & RETURN
  end repeat
  if voidp(glob) then
    put str
    return 
  end if
  gc = glob.managers.config.game
  repeat with x = 1 to 5
    repeat with y = 1 to 10
      lvl = glob.managers.config.getLevelData([#stage: x, #level: y, #mode: #career])
      str = str & "--level " & x & "-" & y & "---" & RETURN
      str = str & lvl.text.succeed & RETURN
      str = str & lvl.text.fail & RETURN
    end repeat
  end repeat
  repeat with pup in glob.managers.config.powerups
    repeat with puplevel in pup.levels
      str = str & "--powerup: " & puplevel.name & "---" & RETURN
      str = str & puplevel.description & RETURN
    end repeat
  end repeat
  put str
end

on voidRefs obj
  if obj.ilk <> #instance then
    return 
  end if
  repeat with i = 1 to obj.count
    obj.setProp(obj.getPropAt(i), VOID)
  end repeat
end

on makeFlosAnimData tilewidth
  if voidp(tilewidth) then
    tilewidth = 37
  end if
  inc = integer(float(tilewidth) / 3)
  n = getNodeData("flo")
  a = getAnimData(n.children["body"].frames)
  addnums = [0, inc + 3, 2 * inc]
  repeat with s in ["_0", "_1", "_carry_1", "_carry_2"]
    d = a["down" & s]
    addnum = 1
    repeat with i in d
      i.loc[2] = i.loc[2] + addnums[addnum]
      addnum = addnum + 1
    end repeat
    d = a["up" & s]
    addnum = 1
    repeat with i in d
      i.loc[2] = i.loc[2] - addnums[addnum]
      addnum = addnum + 1
    end repeat
    d = a["left" & s]
    addnum = 1
    repeat with i in d
      i.loc[1] = i.loc[1] - addnums[addnum]
      addnum = addnum + 1
    end repeat
    d = a["right" & s]
    addnum = 1
    repeat with i in d
      i.loc[1] = i.loc[1] + addnums[addnum]
      addnum = addnum + 1
    end repeat
  end repeat
  b = getAnimData(n.children["legs"].frames)
  repeat with s in ["_0", "_1"]
    d = b["down" & s]
    addnum = 1
    repeat with i in d
      i.loc[2] = i.loc[2] + addnums[addnum]
      addnum = addnum + 1
    end repeat
    d = b["up" & s]
    addnum = 1
    repeat with i in d
      i.loc[2] = i.loc[2] - addnums[addnum]
      addnum = addnum + 1
    end repeat
    d = b["left" & s]
    addnum = 1
    repeat with i in d
      i.loc[1] = i.loc[1] - addnums[addnum]
      addnum = addnum + 1
    end repeat
    d = b["right" & s]
    addnum = 1
    repeat with i in d
      i.loc[1] = i.loc[1] + addnums[addnum]
      addnum = addnum + 1
    end repeat
  end repeat
  c = []
  repeat with num = 1 to 2
    x = getAnimData(n.children["payload_" & num].frames)
    d = x["down"]
    addnum = 1
    repeat with i in d
      i.loc[2] = i.loc[2] + addnums[addnum]
      addnum = addnum + 1
    end repeat
    d = x["up"]
    addnum = 1
    repeat with i in d
      i.loc[2] = i.loc[2] - addnums[addnum]
      addnum = addnum + 1
    end repeat
    d = x["left"]
    addnum = 1
    repeat with i in d
      i.loc[1] = i.loc[1] - addnums[addnum]
      addnum = addnum + 1
    end repeat
    d = x["right"]
    addnum = 1
    repeat with i in d
      i.loc[1] = i.loc[1] + addnums[addnum]
      addnum = addnum + 1
    end repeat
    c.add(x)
  end repeat
  f = []
  repeat with num = 1 to 2
    x = getAnimData(n.children["food_" & num].frames)
    d = x["down"]
    addnum = 1
    repeat with i in d
      i.loc[2] = i.loc[2] + addnums[addnum]
      addnum = addnum + 1
    end repeat
    d = x["up"]
    addnum = 1
    repeat with i in d
      i.loc[2] = i.loc[2] - addnums[addnum]
      addnum = addnum + 1
    end repeat
    d = x["left"]
    addnum = 1
    repeat with i in d
      i.loc[1] = i.loc[1] - addnums[addnum]
      addnum = addnum + 1
    end repeat
    d = x["right"]
    addnum = 1
    repeat with i in d
      i.loc[1] = i.loc[1] + addnums[addnum]
      addnum = addnum + 1
    end repeat
    f.add(x)
  end repeat
  member("flos_anim_data").text = devoid(string([#body: a, #legs: b, #payload: c, #food: f]))
end

on Keys pl
  ls = []
  repeat with i = 1 to pl.count
    p = pl.getPropAt(i)
    ls.add(p)
  end repeat
  return ls
end

on renameppl Prefix
  repeat with clr in ["red", "blue", "green", "purple", "yellow"]
    cstname = "gfx.people." & clr
    charcount = clr.char.count
    repeat with num = 1 to the number of castMembers of castLib cstname
      m = member(num, cstname)
      nm = m.name
      if nm.char[1..charcount] = clr then
        delete nm.char[1..charcount]
        nm = Prefix & nm
        m.name = nm
      end if
    end repeat
  end repeat
end

on rmcolornames
  repeat with clr in ["red", "blue", "green", "purple", "yellow"]
    cstname = "gfx.people." & clr
    charcount = clr.char.count
    repeat with num = 1 to the number of castMembers of castLib cstname
      m = member(num, cstname)
      nm = m.name
      indx = offset("_" & clr & "_", nm)
      if indx = 0 then
        next repeat
      end if
      delete nm.char[indx..indx + charcount]
      m.name = nm
    end repeat
  end repeat
end

on testcolorbonus
  n = glob.managers.game.room.tables[1].chairs[1].views[1].node
  glob.managers.view.showChairBonus(n, [#color: #red, #score: 5000, #multiplier: 4])
end

on testTip num
  if voidp(num) then
    num = 1000
  end if
  n = glob.managers.game.room.tables[1].views[1].tipnode
  glob.managers.view.showTip(n, [#total: num, #extra_star: 1])
end

on asType var, typ
  case typ of
    #string:
      ret = string(var)
    #integer:
      if var = EMPTY then
        var = 0
      end if
      ret = integer(var)
    #float:
      if var = EMPTY then
        var = 0
      end if
      ret = float(var)
    #symbol:
      ret = symbol(var)
    #number:
      if var = EMPTY then
        var = 0
      end if
      var = string(var)
      if offset(".", var) = 0 then
        ret = integer(var)
      else
        ret = float(var)
      end if
  end case
  if not ilk(ret, typ) then
    complain("problem in asType")
    x = 0
  end if
  return ret
end

on putchars n1, n2
  str = EMPTY
  repeat with i = n1 to n2
    str = str & numToChar(i)
  end repeat
  put str
end

on callAndPutMS h, t, Args
  m = the milliSeconds
  ret = call(h, t, Args)
  put t & "." & h & "(" & Args & ") took " & the milliSeconds - m & " MS"
  return ret
end

on listmembers num
  num = num * 1000
  repeat with i = 1 to the number of castLibs
    lib = castLib(i)
    put "____________castlib " & lib.name & "______________"
    sz = 0
    repeat with j = 1 to the number of castMembers of castLib i
      mem = member(j, i)
      if mem.type = #empty then
        next repeat
      end if
      sz = sz + mem.size
      if mem.size > num then
        put mem.name && mem.size
      end if
    end repeat
    put "___________________________________________________"
    put "total size = " & string(sz)
    put "---------------------------------------------------"
  end repeat
end

on convtext memname, cst
  if ilk(memname) = #member then
    mem = memname
  else
    mem = member(memname, cst)
  end if
  conv2BM(mem)
  makeAlphaMember(mem)
end

on makeAlphaMember mem
  mem2 = member(mem.number + 1, mem.castLibNum)
  if mem2.type <> #empty then
    alert("clear a space after the cast member you want to get alpha from")
    return 
  end if
  mem2 = new(#bitmap, mem2)
  mem2.name = "mask_" & mem.name
  mem2.image = mem.image.extractAlpha()
  mem2.regPoint = mem.regPoint
end

on conv2BM tmem
  temp = tmem.image.duplicate()
  nm = tmem.name
  tmem = new(#bitmap, tmem)
  tmem.trimWhiteSpace = 0
  tmem.name = nm
  tmem.image = temp
  tmem.regPoint = point(0, 0)
end

on collapseCast clibname
  mems = []
  repeat with i = 1 to the number of castMembers of castLib clibname
    m = member(i, clibname)
    if m.type <> #empty then
      mems.add(m)
    end if
  end repeat
  repeat with m in mems
    m.move()
  end repeat
end

on addSpacers clibname
  mems = []
  repeat with i = 1 to the number of castMembers of castLib clibname
    m = member(i, clibname)
    if m.type <> #empty then
      mems.add(m)
    end if
  end repeat
  indx = 1
  repeat with m in mems
    m.move(indx)
    indx = indx + 2
  end repeat
end

on helpDirectorFindCastlibs
  mp = the moviePath
  castLib("gfx.room").fileName = mp & "gfx\gfx.room.cct"
  castLib("gfx.people").fileName = mp & "gfx\gfx.people.cct"
  castLib("gfx.screens").fileName = mp & "gfx\gfx.screens.cct"
  castLib("config").fileName = mp & "config\config.cct"
  castLib("gfx.people.red").fileName = mp & "gfx\gfx.people.red.cct"
  castLib("gfx.people.blue").fileName = mp & "gfx\gfx.people.blue.cct"
  castLib("gfx.people.green").fileName = mp & "gfx\gfx.people.green.cct"
  castLib("gfx.people.purple").fileName = mp & "gfx\gfx.people.purple.cct"
  castLib("gfx.people.yellow").fileName = mp & "gfx\gfx.people.orange.cct"
  castLib("sfx").fileName = mp & "snd\sfx.cct"
  castLib("gfx.family").fileName = mp & "gfx\gfx.tiki.cct"
  castLib("gfx.finedining").fileName = mp & "gfx\gfx.seafood.cct"
  castLib("gfx.chezflos").fileName = mp & "gfx\gfx.chez.cct"
  castLib("gfx.goddess").fileName = mp & "gfx\gfx.goddess.cct"
  castLib("gfx.people.none").fileName = mp & "gfx\gfx.people.none.cct"
  castLib("gfx.diner").fileName = mp & "gfx\gfx.diner.cct"
  castLib("doodles").fileName = mp & "gfx\doodles.cct"
  castLib("gfx.numbers").fileName = mp & "gfx\gfx.numbers.cct"
  castLib("scripts.sprite").fileName = mp & "ext\sprite.cct"
  castLib("scripts.mode").fileName = mp & "ext\mode.cct"
  castLib("scripts.events").fileName = mp & "ext\events.cct"
  castLib("scripts.main").fileName = mp & "ext\main.cct"
  castLib("scripts.layout").fileName = mp & "ext\layout.cct"
  castLib("scripts.xml").fileName = mp & "ext\xml.cct"
  castLib("scripts.config").fileName = mp & "ext\conf.cct"
  castLib("scripts.tutorial").fileName = mp & "ext\tutorial.cct"
  castLib("scripts.game").fileName = mp & "ext\game.cct"
  castLib("scripts.anim").fileName = mp & "ext\anim.cct"
  castLib("leveltext").fileName = mp & "ext\leveltext.cct"
  castLib("gfx.intro").fileName = mp & "gfx\gfx.intro.cct"
  member("gamelab_splash.dcr", "gfx.intro").fileName = mp & "gamelab_splash.dcr"
end

on makeAlphaAll clibname
  if voidp(clibname) then
    clibname = "dynamic"
  end if
  mems = []
  repeat with i = 1 to the number of castMembers of castLib clibname
    m = member(i, clibname)
    if (m.type = #bitmap) and (m.name <> EMPTY) and not (m.name contains "mask_") then
      mems.add(m)
    end if
  end repeat
  repeat with m in mems
    makeAlphaMember(m)
  end repeat
  collapseCast(clibname)
end

on unionset lists
  bigset = []
  repeat with i in lists
    repeat with element in i
      if bigset.getOne(element) = 0 then
        bigset.add(element)
      end if
    end repeat
  end repeat
  return bigset
end

on overlaps list1, list2
  repeat with i in list1
    if list2.getOne(i) then
      return 1
    end if
  end repeat
  return 0
end

on checkCallback cb
  if not ilk(cb, #propList) then
    return 
  end if
  obj = cb[#target]
  if not obj.objectp then
    return 
  end if
  h = cb[#handler]
  if not h.symbolp then
    return 
  end if
  if obj.handlers().getOne(h) = 0 then
    return 
  end if
  return 1
end

on dequote str
  if str.char[1] = QUOTE then
    delete str.char[1]
  end if
  if str.char[2] = QUOTE then
    delete str.char[1..2]
  end if
  if str.char[str.length] = QUOTE then
    delete str.char[str.length]
  end if
  return str
end

on commafy str
  str = dequote(str)
  ct = str.length
  div = (ct - 1) / 3
  repeat while div
    put "," before str.char[ct - 2]
    ct = ct - 3
    div = (ct - 1) / 3
  end repeat
  return str
end

on fixCompression
  repeat with i = 1 to the number of castLibs
    repeat with h = 1 to the number of castMembers of castLib i
      m = member(h, i)
      if m.type = #bitmap then
        m.imageCompression = #standard
        m.imageQuality = 100
      end if
    end repeat
  end repeat
end

on pp L
  put pp2(L, 0)
end

on pp2 L, indent
  if voidp(indent) then
    indent = 0
  end if
  if voidp(L) then
    return "void"
  end if
  r = EMPTY
  case ilk(L) of
    #propList:
      r = r & pindent(indent) & "["
      if L.count = 0 then
        r = r & ":]"
      else
        r = r & RETURN
        repeat with i = 1 to L.count
          r = r & pindent(indent)
          a = pp2(L.getPropAt(i), 0)
          r = r & a & ": "
          if the last char in a = RETURN then
            r = r & pindent(indent)
          end if
          a = pp2(L[i], indent + 2)
          r = r & a
          if the last char in a = RETURN then
            r = r & pindent(indent)
          end if
          if i < L.count then
            r = r & ", " & RETURN
          end if
        end repeat
        r = r & "]" & RETURN
      end if
    #list:
      r = r & "["
      if L.count = 0 then
        r = r & "]"
      else
        r = r & RETURN
        repeat with i = 1 to L.count
          r = r & pindent(indent)
          a = pp2(L[i], indent + 2)
          r = r & a
          if the last char in a = RETURN then
            r = r & pindent(indent)
          end if
          if i < L.count then
            r = r & ", " & RETURN
          end if
        end repeat
        r = r & "]" & RETURN
      end if
    #string:
      r = r & QUOTE & L & QUOTE
    #symbol:
      r = r & "#" & L
    otherwise:
      r = r & L
  end case
  return r
end

on pindent n
  r = EMPTY
  repeat with i = 1 to n
    r = r & " "
  end repeat
  return r
end
