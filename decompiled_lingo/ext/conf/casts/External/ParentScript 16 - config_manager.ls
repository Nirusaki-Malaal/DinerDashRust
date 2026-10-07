property overall, customer, group, game, powerups, gfx, useExtConfigs, author, xparser, addTableName, addTableDesc, snackIDs, initialized, sitterRegPoints

on new me
  author = the runMode contains "Author"
  snackIDs = [#papadom, #somosa, #crab, #bread]
  m = member("refresh_flag", "scripts.config")
  if m.number = -1 then
    m = new(#text, castLib("scripts.config"))
    m.text = "1"
    m.name = "refresh_flag"
  end if
  if integer(m.text) then
    refreshConfigs()
    m.text = "0"
  end if
  if author then
    baRegister("gameLab", 2298015)
    xparser = script("xml.parser").new()
  end if
  addTableName = "new table"
  addTableDesc = "add a new table to the restaurant"
  gfx = [:]
  dstates = [:]
  repeat with i = 1 to 4
    ds = [:]
    repeat with j = 1 to 3
      if j = 2 then
        nms = ["remove_2"]
      else
        nms = ["remove_" & j & "a", "remove_" & j & "b", "remove_" & j & "c"]
      end if
      repeat with n in nms
        if j >= i then
          ds[n] = 1
          next repeat
        end if
        ds[n] = 0
      end repeat
    end repeat
    dstates["state_" & i] = ds
  end repeat
  tiki = ["state_1": ["tiki_2": 0, "tiki_3": 0, "tiki_4": 0], "state_2": ["tiki_2": 1, "tiki_3": 0, "tiki_4": 0], "state_3": ["tiki_2": 1, "tiki_3": 1, "tiki_4": 0], "state_4": ["tiki_2": 1, "tiki_3": 1, "tiki_4": 1]]
  seafood = ["state_1": ["bg_seafood_2": 0, "bg_seafood_3": 0, "bg_seafood_4": 0], "state_2": ["bg_seafood_2": 1, "bg_seafood_3": 0, "bg_seafood_4": 0], "state_3": ["bg_seafood_2": 1, "bg_seafood_3": 1, "bg_seafood_4": 0], "state_4": ["bg_seafood_2": 1, "bg_seafood_3": 1, "bg_seafood_4": 1]]
  chez = ["state_1": ["bg_chez_2": 0, "bg_chez_3": 0, "bg_chez_4a": 0, "bg_chez_4b": 0], "state_2": ["bg_chez_2": 1, "bg_chez_3": 0, "bg_chez_4a": 0, "bg_chez_4b": 0], "state_3": ["bg_chez_2": 1, "bg_chez_3": 1, "bg_chez_4a": 0, "bg_chez_4b": 0], "state_4": ["bg_chez_2": 1, "bg_chez_3": 1, "bg_chez_4a": 1, "bg_chez_4b": 1]]
  goddess = [:]
  bg = ["bg_1", "bg_2"]
  crack1 = ["crack_1_a", "crack_1_b", "crack_1_c"]
  crack2 = []
  crack3 = []
  repeat with i in ["a", "b", "c", "d"]
    crack2.add("crack_2_" & i)
    crack3.add("crack_3_" & i)
  end repeat
  st = [:]
  k = unionset([bg, crack1, crack2, crack3])
  repeat with i in k
    st[i] = 0
  end repeat
  repeat with i in ["state_1", "state_2", "state_3", "state_4", "state_5"]
    goddess[i] = st.duplicate()
  end repeat
  repeat with i in crack1
    goddess["state_2"][i] = 1
  end repeat
  repeat with i in crack2
    goddess["state_3"][i] = 1
  end repeat
  repeat with i in crack3
    goddess["state_4"][i] = 1
  end repeat
  repeat with i = 1 to 4
    goddess["state_" & i]["bg_2"] = 1
  end repeat
  goddess["state_5"]["bg_2"] = 1
  gfx["diner"] = [#nodeID: "bg_diner", #states: dstates]
  gfx["family"] = [#nodeID: "bg_tiki", #states: tiki]
  gfx["finedining"] = [#nodeID: "bg_seafood", #states: seafood]
  gfx["chezflos"] = [#nodeID: "bg_chez", #states: chez]
  gfx["goddess"] = [#nodeID: "bg_goddess", #states: goddess]
  m = member("sitter_regpoints", "scripts.layout")
  if m.number <> -1 then
    sitterRegPoints = value(m.text)
  end if
  if voidp(sitterRegPoints) then
    sitterRegPoints = [:]
  end if
  return me
end

on init me
  if initialized then
    return 
  end if
  initialized = 1
  m1 = the milliSeconds
  me.getGlobal()
  me.getCustomer()
  me.getPowerups()
  game = [:]
  me.getCareer()
  me.getEndless()
end

on getValueIfUnchanged me, str
  useExt = 0
  if author then
    filedate = baFileDate(the moviePath & "config/" & str & ".xml", "mm/dd/yy", ":hh:nn:ss")
    useExt = filedate <> member(str & "_date").text
    member(str & "_date").text = filedate
    if (str = "endless") or (str = "career") then
      useExt = useExt or me.getConfigCastChanged()
    end if
  end if
  if not useExt then
    return me.loadData(str)
  end if
end

on getConfigCastChanged me
  changed = 0
  if author then
    fn = castLib("config").fileName
    filedate = baFileDate(fn, "mm/dd/yy", ":hh:nn:ss")
    m = member("config_date", "scripts.config")
    if m.number = -1 then
      m = new(#text, castLib("scripts.config"))
      m.name = "config_date"
    end if
    changed = filedate <> m.text
    m.text = filedate
  end if
  return changed
end

on loadData me, str
  m = member(str & "_dump", "scripts.config")
  if m.number = -1 then
    return 
  end if
  return value(devoid(m.text))
end

on saveData me, str, data
  m = member(str & "_dump", "scripts.config")
  if m.number = -1 then
    m = new(#text, castLib("scripts.config"))
    m.name = str & "_dump"
  end if
  m.text = string(data)
end

on parseXML me, str
  id = getNetText("config/" & str & ".xml")
  xmltext = netTextResult(id)
  xparser.parseString(xmltext)
end

on getGlobal me
  overall = me.getValueIfUnchanged("global")
  if overall.listp then
    return 
  end if
  me.parseXML("global")
  gbl = xparser[1]
  att = gbl.attributeValue
  overall = [:]
  repeat with i = 1 to att.count
    overall[att.getPropAt(i)] = integer(att[i])
  end repeat
  pts = [:]
  apnode = gbl["actionPoints"]
  att = apnode.attributeValue
  repeat with i in apnode.attributeName
    pts[symbol(i)] = integer(att[i])
  end repeat
  overall[#actionPoints] = pts
  tb = [:]
  tbnode = gbl["timebomb"]
  att = tbnode.attributeValue
  repeat with i in tbnode.attributeName
    tb[symbol(i)] = integer(att[i])
  end repeat
  overall[#timebomb] = tb
  me.saveData("global", overall)
end

on getPowerups me
  powerups = me.getValueIfUnchanged("powerups")
  if powerups.listp then
    return 
  end if
  me.parseXML("powerups")
  pupnodelist = xparser["powerups"]
  powerups = [:]
  repeat with pupnode in pupnodelist
    atts = pupnode.attributeValue
    pup = [:]
    repeat with k in pupnode.attributeName
      pup[symbol(k)] = atts[k]
    end repeat
    lvls = []
    puplevelnodes = pupnode["powerupLevels"]
    repeat with i = 1 to puplevelnodes.count
      lvlnode = puplevelnodes[i]
      lvl = [:]
      repeat with k in lvlnode.attributeName
        lvl[symbol(k)] = lvlnode.attributeValue[k]
      end repeat
      props = [:]
      numprops = lvlnode["numProps"].attributeValue
      klist = lvlnode["numProps"].attributeName
      repeat with k in klist
        props[symbol(k)] = asType(numprops[k], #number)
      end repeat
      lvl[#properties] = props
      lvl[#level] = i
      lvl[#type] = symbol(pup.id)
      lvls.add(lvl)
    end repeat
    pup[#levels] = lvls
    powerups[symbol(pup.id)] = pup
  end repeat
  me.saveData("powerups", powerups)
end

on getCustomer me
  gc = me.getValueIfUnchanged("customer")
  if gc.listp then
    customer = gc.customer
    group = gc.group
    return 
  end if
  me.parseXML("customer")
  cust = xparser["customerConfig"]
  customer = [:]
  repeat with i in cust["customers"]
    att = i.attributeValue
    customer[att["id"]] = [#color: symbol(att["color"])]
  end repeat
  activeStates = [#deciding, #eating, #scoring, #leaving]
  passiveStates = [#waiting, #ready, #ordered, #check]
  group = [:]
  gdefs = cust["groupDefaults"]
  att = gdefs.attributeValue
  gd = [:]
  repeat with k in gdefs.attributeName
    gd[symbol(k)] = asType(att[k], #number)
  end repeat
  defstates = [:]
  repeat with i in gdefs["groupStates"]
    att = i.attributeValue
    gs = [:]
    repeat with k in i.attributeName
      if k = "id" then
        next repeat
      end if
      gs[symbol(k)] = integer(att[k])
    end repeat
    gs[#minMS] = gd.minMS
    defstates[symbol(att["id"])] = gs
  end repeat
  gd[#stateData] = defstates
  group[#defaults] = gd
  gtypes = [:]
  repeat with tp in cust["groupTypes"]
    gt = gd.duplicate()
    att = tp.attributeValue
    repeat with k in tp.attributeName
      if k = "id" then
        next repeat
        next repeat
      end if
      if k = "graphicType" then
        gt[#graphicType] = att[k]
        next repeat
      end if
      gt[symbol(k)] = asType(att[k], #number)
    end repeat
    gtypes[att["id"]] = gt
    override_states = tp["groupStates"]
    if att["id"] = "boy" then
      nothing()
    end if
    if not voidp(override_states) then
      repeat with huh = 1 to override_states.count
        os = override_states.child[huh]
        os_att = os.attributeValue
        gs = [:]
        repeat with k in os.attributeName
          if k = "id" then
            next repeat
          end if
          gs[k] = integer(os_att[k])
        end repeat
        gs[#minMS] = gd.minMS
        gt.stateData[symbol(os_att["id"])] = gs
      end repeat
      nothing()
    end if
    slowness = gt.slowness
    patience = gt.patience
    if (slowness = 1.0) and (patience = 1.0) then
      next repeat
    end if
    repeat with st in activeStates
      statelist = gt.stateData[st]
      repeat with modnum = 1 to statelist.count
        statelist[modnum] = integer(statelist[modnum] * slowness)
      end repeat
    end repeat
    repeat with st in passiveStates
      statelist = gt.stateData[st]
      repeat with modnum = 1 to statelist.count
        statelist[modnum] = integer(statelist[modnum] * patience)
      end repeat
    end repeat
  end repeat
  group[#types] = gtypes
  me.saveData("customer", [#group: group, #customer: customer])
end

on getCareer me
  game[#career] = me.getValueIfUnchanged("career")
  if game.career.listp then
    return 
  end if
  me.parseXML("career")
  career = xparser["careerMode"]
  stages = [:]
  repeat with stg_node in career["stages"]
    att = stg_node.attributeValue
    st = [:]
    repeat with k in stg_node.attributeName
      val = att[k]
      if k = "id" then
        val = symbol(val)
      else
        if k = "numStars" then
          val = integer(val)
        end if
      end if
      st[symbol(k)] = val
    end repeat
    lvls = []
    repeat with lvl_node in stg_node["levels"]
      lvl = [:]
      lvl_att = lvl_node.attributeValue
      repeat with k in lvl_node.attributeName
        if k = "layoutFile" then
          next repeat
        end if
        if (k = "bgState") or (k = "doodle") then
          lvl[symbol(k)] = lvl_att[k]
          next repeat
        end if
        lvl[symbol(k)] = integer(lvl_att[k])
      end repeat
      memname = lvl_node.attributeValue["layoutFile"]
      m = member(memname, "config")
      if m.number = -1 then
        complain("no leveldata member named " & memname & " in config cast")
        m = member("thislevel", "config")
      end if
      lvl[#layout] = getLevelLayout(m.text)
      pod_node = lvl_node["podium"]
      if not voidp(pod_node) then
        podls = [:]
        repeat with pod_att in pod_node.attributeName
          podls[symbol(pod_att)] = integer(pod_node.attributeValue[pod_att])
        end repeat
        lvl[#podium] = podls
      end if
      upg_node = lvl_node["powerups"]
      if not voidp(upg_node) then
        sugs = [:]
        repeat with k in upg_node.attributeName
          pup_type = symbol(k)
          pup_level = integer(upg_node.attributeValue[k])
          pup = me.getPowerupConfig(pup_type, pup_level)
          if voidp(pup) then
            complain("no powerup '" & pup_type & "' lvl '" & pup_level, " in powerup.xml")
            next repeat
          end if
          sugs[pup_type] = pup
        end repeat
        lvl[#powerups] = sugs
      end if
      if not voidp(lvl[#snackPercent]) then
        snacks = []
        snack_nodes = lvl_node["snacks"]
        if not voidp(snack_nodes) then
          repeat with sn in snack_nodes
            snid = symbol(sn.getText())
            if snackIDs.getOne(snid) = 0 then
              complain(snid & " not one of currently available snacks: " & string(snackIDs))
              next repeat
            end if
            if snacks.getOne(snid) = 0 then
              snacks.add(snid)
            end if
          end repeat
        end if
        lvl[#snacks] = snacks
      end if
      grps = []
      repeat with gr_node in lvl_node["groups"]
        gr = [:]
        gr_att = gr_node.attributeValue
        gr[#queueTimeMS] = integer(gr_att.queueTimeMS)
        gr[#type] = gr_att["groupType"]
        cs = []
        repeat with cid in gr_node["customerIDs"]
          cs.add(customer[cid.getText()])
        end repeat
        gr[#customers] = cs
        grps.add(gr)
      end repeat
      lvl[#groups] = grps
      txt = [:]
      repeat with txtnode in lvl_node["text"]
        txt[symbol(txtnode.name)] = txtnode.getText()
      end repeat
      lvl[#text] = txt
      lvls.add(lvl)
      me.saveData("career_" & stages.count + 1 & "_" & lvls.count, lvl)
    end repeat
    st[#numLevels] = lvls.count
    st[#introText] = stg_node["introText"].getText()
    stages[symbol(att["id"])] = st
  end repeat
  game.career = [#stages: stages, #numStages: stages.count]
  me.saveData("career", game.career)
end

on getEndless me
  game[#endless] = me.getValueIfUnchanged("endless")
  if game.endless.listp then
    return 
  end if
  me.parseXML("endless")
  endls = [:]
  enode = xparser["endlessMode"]
  repeat with stagenode in enode["endlessStages"]
    estage = [:]
    att = stagenode.attributeValue
    repeat with k in stagenode.attributeName
      if (k = "name") or (k = "id") then
        estage[symbol(k)] = att[k]
        next repeat
      end if
      estage[symbol(k)] = asType(att[k], #number)
    end repeat
    lvls = []
    repeat with lvl_node in stagenode["endlessLevels"]
      lvl = [:]
      lvl_att = lvl_node.attributeValue
      repeat with k in lvl_node.attributeName
        if (k = "layoutFile") or (k = "id") then
          next repeat
        end if
        lvl[symbol(k)] = asType(lvl_att[k], #number)
      end repeat
      repeat with rampprop in [#difficultyIncreaseMS, #startingTimeMultiplier, #timeMultiplierDecrease, #timeMultiplierMinimum]
        if voidp(lvl[rampprop]) then
          lvl[rampprop] = estage[rampprop]
        end if
      end repeat
      memname = lvl_node.attributeValue["layoutFile"]
      m = member(memname, "config")
      if m.number = -1 then
        complain("no leveldata member named " & memname & " in config cast")
        m = member("thislevel", "config")
      end if
      lvl[#layout] = getLevelLayout(m.text)
      pod_node = lvl_node["podium"]
      if not voidp(pod_node) then
        podls = [:]
        repeat with pod_att in pod_node.attributeName
          podls[symbol(pod_att)] = integer(pod_node.attributeValue[pod_att])
        end repeat
        lvl[#podium] = podls
      end if
      sugs = [:]
      upg_node = lvl_node["powerups"]
      if not voidp(upg_node) then
        repeat with k in upg_node.attributeName
          pup_type = symbol(k)
          pup_level = integer(upg_node.attributeValue[k])
          pup = me.getPowerupConfig(pup_type, pup_level)
          if voidp(pup) then
            complain("no powerup '" & pup_type & "' lvl '" & pup_level, " in powerup.xml")
            next repeat
          end if
          sugs[pup_type] = pup
        end repeat
      end if
      lvl[#powerups] = sugs
      if not voidp(lvl[#snackPercent]) then
        snacks = []
        snack_nodes = lvl_node["snacks"]
        if not voidp(snack_nodes) then
          repeat with sn in snack_nodes
            snid = symbol(sn.getText())
            if snackIDs.getOne(snid) = 0 then
              complain(snid & " not one of currently available snacks: " & string(snackIDs))
              next repeat
            end if
            if snacks.getOne(snid) = 0 then
              snacks.add(snid)
            end if
          end repeat
        end if
        lvl[#snacks] = snacks
      end if
      tugs = []
      last_layout = lvl.layout
      tugnodes = lvl_node["tableUpgrades"]
      if not voidp(tugnodes) then
        repeat with ugnode in lvl_node["tableUpgrades"]
          tugname = ugnode.getText()
          m = member(tugname, "config")
          if m.number = -1 then
            complain("no leveldata member named " & tugname & " in config cast")
            m = member("thislevel", "config")
          end if
          newlayout = getLevelLayout(m.text)
          tug = getAddedTable(last_layout, newlayout)
          if voidp(tug) then
            complain("problem using layout file " & tugname & " as table upgrade")
            exit repeat
          end if
          tugs.add(tug)
          last_layout = newlayout
        end repeat
      end if
      lvl[#tableUpgrades] = tugs
      slvls = []
      slvl_nodes = lvl_node["subLevels"]
      pup_keeptrack = lvl.powerups.duplicate()
      last_goal = 0
      if not voidp(slvl_nodes) then
        repeat with subnode in slvl_nodes
          slvl = [:]
          slvl[#goal] = integer(subnode.attributeValue["goal"]) + last_goal
          last_goal = slvl[#goal]
          grps = []
          totfreq = 0
          repeat with gr_node in subnode["randomGroups"]
            gr = [:]
            gr_att = gr_node.attributeValue
            qtms = gr_att[#queueTimeMS]
            if voidp(qtms) then
              put "no queueTimeMS in an endless level"
              next repeat
            end if
            gr[#queueTimeMS] = integer(gr_att.queueTimeMS)
            freq = integer(gr_att["frequency"])
            gr[#Frequency] = freq
            totfreq = totfreq + freq
            gr[#type] = gr_att["groupType"]
            grps.add(gr)
          end repeat
          slvl[#groups] = grps
          slvl[#totalGroupFrequency] = totfreq
          ugnodes = subnode["upgrades"]
          if not voidp(ugnodes) then
            ugs = []
            repeat with ugnode in ugnodes
              pup_type = symbol(ugnode.getText())
              mypup = pup_keeptrack[pup_type]
              if voidp(mypup) then
                lvlnum = 1
              else
                lvlnum = mypup.level + 1
              end if
              if pup_type = #table then
                if lvlnum > lvl.tableUpgrades.count then
                  complain("table upgrade offered " & lvlnum & " times but only " & lvl.tableUpgrades.count & " layout files defined ")
                  return 
                end if
                pup = [#type: #table, #level: lvlnum, #name: addTableName, #description: addTableDesc, #properties: lvl.tableUpgrades[lvlnum]]
              else
                pup = me.getPowerupConfig(pup_type, lvlnum)
                if voidp(pup) then
                  complain("no powerup '" & pup_type & "' lvl '" & lvlnum, " in powerup.xml")
                  next repeat
                end if
              end if
              pup_keeptrack[pup_type] = pup
              ugs.add(pup)
            end repeat
            slvl[#upgrades] = ugs
          end if
          rcol_nodes = subnode["randomColors"]
          if not voidp(rcol_nodes) then
            rcols = []
            repeat with rcol_node in rcol_nodes
              rcols.add(symbol(rcol_node.getText()))
            end repeat
            slvl[#randomColors] = rcols
          end if
          slvls.add(slvl)
          me.saveData("endsub_" & endls.count + 1 & "_" & lvls.count + 1 & "_" & slvls.count, slvl)
        end repeat
      end if
      lvl[#numSublevels] = slvls.count
      lvls.add(lvl)
      me.saveData("endless_" & endls.count + 1 & "_" & lvls.count, lvl)
    end repeat
    estage[#numLevels] = lvls.count
    endls[symbol(estage.id)] = estage
  end repeat
  game.endless = [#stages: endls, #numStages: endls.count]
  me.saveData("endless", game.endless)
end

on getConfig
  return overall.duplicate()
end

on getStageData me, Args
  return game[Args.mode].stages[Args.stage]
end

on getLevelData me, Args
  d = me.loadData(Args.mode & "_" & Args.stage & "_" & Args.level)
  return d
end

on getSublevelData me, Args
  d = me.loadData("endsub_" & Args.stage & "_" & Args.difficulty & "_" & Args.sublevel)
  return d
end

on getPowerupConfig me, type, level
  pc = powerups[type]
  if voidp(pc) then
    return 
  end if
  level = max(1, level)
  if pc.levels.count < level then
    return 
  end if
  return pc.levels[level]
end

on getLevels me, st
  if voidp(st) then
    st = #diner
  end if
  if voidp(game) then
    me.init()
  end if
  return game.career.stages[#diner].levels
end
