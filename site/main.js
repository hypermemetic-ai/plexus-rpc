(function(){
var cv=document.getElementById('c'),gl=cv.getContext('webgl',{antialias:false,alpha:false}),btn=document.getElementById('play');
if(!gl){document.documentElement.className+=' nogl';btn.hidden=true;document.getElementById('rate').hidden=true;return;}
function sh(t,s){var o=gl.createShader(t);gl.shaderSource(o,s);gl.compileShader(o);if(!gl.getShaderParameter(o,gl.COMPILE_STATUS))throw new Error(gl.getShaderInfoLog(o));return o;}
function prog(vs,fs,names){var p=gl.createProgram();gl.attachShader(p,sh(gl.VERTEX_SHADER,vs));gl.attachShader(p,sh(gl.FRAGMENT_SHADER,fs));gl.bindAttribLocation(p,0,'p');gl.linkProgram(p);
  if(!gl.getProgramParameter(p,gl.LINK_STATUS))throw new Error(gl.getProgramInfoLog(p));var u={p:p};names.forEach(function(n){u[n]=gl.getUniformLocation(p,n);});return u;}
// The web is geometry: six star polygons of twenty-four points, each drawn as its own ink cells, and each star
// flipping what the ones before it left. So the card fills them and no pixel loops.
// It is drawn into a texture at twice the screen's size (which smooths its edges), with the dot in a second channel;
// three small passes make a soft copy at a quarter size. All of that happens only when the web's time or size changes.
// The screen is then that texture in the page's colours, and over it one small quad for each pane of glass.
var VQ='attribute vec2 p;varying vec2 v;void main(){v=p*.5+.5;gl_Position=vec4(p,0.,1.);}',VP='attribute vec2 p;void main(){gl_Position=vec4(p,0.,1.);}';
var flat=prog(VP,'precision mediump float;uniform vec4 uC;void main(){gl_FragColor=uC;}',['uC']);
var down=prog(VQ,'precision mediump float;varying vec2 v;uniform sampler2D uS;uniform vec2 uO;void main(){gl_FragColor=(texture2D(uS,v+uO)+texture2D(uS,v-uO)+texture2D(uS,v+vec2(uO.x,-uO.y))+texture2D(uS,v+vec2(-uO.x,uO.y)))*.25;}',['uS','uO']);
var show=prog(VQ,'precision mediump float;varying vec2 v;uniform sampler2D uA;uniform vec3 uInk,uPaper,uAcc;void main(){vec4 m=texture2D(uA,v);gl_FragColor=vec4(mix(mix(uPaper,uInk,m.r),uAcc,m.g),1.);}',['uA','uInk','uPaper','uAcc']);
var glassP=prog(VP,"precision highp float;\nuniform sampler2D uA,uB;uniform vec2 uRes;uniform float uPx;uniform vec3 uInk,uPaper,uAcc;uniform vec4 uC,uP;\nvec3 col(vec2 m){return mix(mix(uPaper,uInk,m.x),uAcc,m.y);}\n// One pane of glass over the web's picture (uA sharp, uB soft). Its middle shows the soft web, tinted; its rim is a\n// lens: the sharp web bent toward the middle, each colour a little further, under a bright edge. Outside it, its shadow.\nvoid main(){\n  vec2 rel=gl_FragCoord.xy-uC.xy,q=abs(rel)-uC.zw+uP.x;\n  float d=length(max(q,0.))+min(max(q.x,q.y),0.)-uP.x;\n  if(d>0.){gl_FragColor=vec4(0.,0.,0.,.3*exp(-d/(12.*uPx)));return;}\n  vec2 n=sign(rel)*((q.x>0.&&q.y>0.)?normalize(q):(q.x>q.y?vec2(1.,0.):vec2(0.,1.)));\n  float e=clamp(1.+d/uP.w,0.,1.),k=e*e*e;vec2 uv=gl_FragCoord.xy/uRes,o=n*uP.w*2.2*k/uRes;\n  vec3 a=k>.004?vec3(col(texture2D(uA,uv-o*.82).rg).r,col(texture2D(uA,uv-o).rg).g,col(texture2D(uA,uv-o*1.2).rg).b):col(texture2D(uA,uv).rg);\n  vec3 c=mix(a,col(texture2D(uB,uv-o).rg),uP.z*(1.-min(1.,k*1.6)));\n  c=mix(c,uPaper,uP.y*(1.-.75*k));\n  float rim=pow(e,7.),lit=.25+.75*abs(dot(n,normalize(vec2(-.55,.83))));\n  c=mix(c,vec3(1.),rim*lit*.75);c*=1.-.35*pow(e,24.)*(1.-lit);\n  gl_FragColor=vec4(c,1.);}\n",['uA','uB','uRes','uPx','uInk','uPaper','uAcc','uC','uP']);
var buf=gl.createBuffer();gl.bindBuffer(gl.ARRAY_BUFFER,buf);gl.enableVertexAttribArray(0);gl.vertexAttribPointer(0,2,gl.FLOAT,false,0,0);
var FULL=new Float32Array([-1,-1,1,-1,-1,1,1,1]),MAXT=Math.min(4096,gl.getParameter(gl.MAX_TEXTURE_SIZE));
function rgb(n){var v=getComputedStyle(document.documentElement).getPropertyValue(n).trim().replace('#','');return [0,2,4].map(function(i){return parseInt(v.substr(i,2),16)/255;});}
function target(w,h){var tx=gl.createTexture();gl.bindTexture(gl.TEXTURE_2D,tx);gl.texImage2D(gl.TEXTURE_2D,0,gl.RGBA,w,h,0,gl.RGBA,gl.UNSIGNED_BYTE,null);
  gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MIN_FILTER,gl.LINEAR);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MAG_FILTER,gl.LINEAR);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_WRAP_S,gl.CLAMP_TO_EDGE);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_WRAP_T,gl.CLAMP_TO_EDGE);
  var fb=gl.createFramebuffer();gl.bindFramebuffer(gl.FRAMEBUFFER,fb);gl.framebufferTexture2D(gl.FRAMEBUFFER,gl.COLOR_ATTACHMENT0,gl.TEXTURE_2D,tx,0);return {tx:tx,fb:fb,w:w,h:h};}
function drop(o){if(o){gl.deleteTexture(o.tx);gl.deleteFramebuffer(o.fb);}}
function into(o){gl.bindFramebuffer(gl.FRAMEBUFFER,o?o.fb:null);gl.viewport(0,0,o?o.w:cv.width,o?o.h:cv.height);}
function quad(a){gl.bufferData(gl.ARRAY_BUFFER,a,gl.DYNAMIC_DRAW);gl.drawArrays(gl.TRIANGLE_STRIP,0,4);}
var d=1,q=1,still=matchMedia('(prefers-reduced-motion:reduce)').matches,on=!still,seen=true,t=8,last=0,drawn=0,raf=0;
var A=null,Hf=null,B=null,B2=null,webT=NaN,uR=1,ink,paper,acc,verts=new Float32Array(1<<18);
var K=[11,10,9,8,7,5],DA=Math.PI/12;
function size(){d=Math.min(devicePixelRatio||1,2)*q;var r=cv.getBoundingClientRect(),w=Math.max(1,Math.round(r.width*d)),h=Math.max(1,Math.round(r.height*d));cv.width=w;cv.height=h;
  uR=Math.min(r.width,r.height)*.46*d;ink=rgb('--ink');paper=rgb('--paper');acc=rgb('--acc');
  var ss=Math.max(1,Math.min(2,3/d,MAXT/Math.max(w,h)));[A,Hf,B,B2].forEach(drop);
  A=target(Math.round(w*ss),Math.round(h*ss));Hf=target(Math.ceil(w/2),Math.ceil(h/2));B=target(Math.ceil(w/4),Math.ceil(h/4));B2=target(B.w,B.h);webT=NaN;draw();}
function disc(n,cx,cy,r,w,h){for(var i=0;i<40;i++){var a=i*Math.PI/20,b=(i+1)*Math.PI/20;verts[n++]=cx/w;verts[n++]=cy/h;verts[n++]=(cx+r*Math.cos(a))/w;verts[n++]=(cy+r*Math.sin(a))/h;verts[n++]=(cx+r*Math.cos(b))/w;verts[n++]=(cy+r*Math.sin(b))/h;}return n;}
function web(){var w=cv.width/2,h=cv.height/2,s=uR*2.02,tiles=Math.ceil((w+uR)/s),n=0,x,i,k;
  var rr=[.93+.07*Math.sin(t*.31),.80+.10*Math.sin(t*.37+1.1),.66+.11*Math.sin(t*.29+2.3),.50+.10*Math.sin(t*.41+3.1),.35+.08*Math.sin(t*.33+4.2),.21+.05*Math.sin(t*.45+5.0)],
      ph=[t*.020,-t*.034,t*.047,-t*.061,t*.078,-t*.095];
  // A star {24/k} is cut by its own edges into rings of cells: its points, then kites, then its inner polygon. A
  // cell is covered w times, w counting in from the points, and is ink when w is odd; a cell's corners lie on the
  // circles of radius d/cos(m*pi/24), m from k at the points to 1 at the inner polygon. Only the ink cells are
  // drawn, each once, so a star costs its own area and no more; the flipping is left to do between the six stars.
  for(x=-tiles;x<=tiles;x++)for(k=0;k<6;k++){var kk=K[k],r=uR*rr[k],dd=r*Math.cos(kk*DA/2),cx=x*s,m,w2;
    for(w2=1;w2<kk;w2+=2){m=kk-w2;var ro=dd/Math.cos((m+1)*DA/2),rs=dd/Math.cos(m*DA/2),ri=m>1?dd/Math.cos((m-1)*DA/2):0,off=ph[k]+(w2-1)*DA/2;
      for(i=0;i<24;i++){var th=off+i*DA,ox=cx+ro*Math.cos(th),oy=ro*Math.sin(th),lx=cx+rs*Math.cos(th-DA/2),ly=rs*Math.sin(th-DA/2),qx=cx+rs*Math.cos(th+DA/2),qy=rs*Math.sin(th+DA/2);
        if(m>1){var ix=cx+ri*Math.cos(th),iy=ri*Math.sin(th);verts[n++]=ox/w;verts[n++]=oy/h;verts[n++]=lx/w;verts[n++]=ly/h;verts[n++]=ix/w;verts[n++]=iy/h;verts[n++]=ox/w;verts[n++]=oy/h;verts[n++]=ix/w;verts[n++]=iy/h;verts[n++]=qx/w;verts[n++]=qy/h;}
        else{verts[n++]=ox/w;verts[n++]=oy/h;verts[n++]=lx/w;verts[n++]=ly/h;verts[n++]=qx/w;verts[n++]=qy/h;}}}
    if(kk%2){var r1=dd/Math.cos(DA/2),o1=ph[k]+(kk-1)*DA/2;for(i=0;i<24;i++){var a=o1+i*DA,b=a+DA;
      verts[n++]=cx/w;verts[n++]=0;verts[n++]=(cx+r1*Math.cos(a))/w;verts[n++]=r1*Math.sin(a)/h;verts[n++]=(cx+r1*Math.cos(b))/w;verts[n++]=r1*Math.sin(b)/h;}}
    // one star at a time, so the flip is between stars
    if(n>verts.length-4096)break;}
  into(A);gl.clearColor(0,0,0,1);gl.clear(gl.COLOR_BUFFER_BIT);gl.useProgram(flat.p);gl.uniform4f(flat.uC,1,1,1,1);
  gl.enable(gl.BLEND);gl.blendFunc(gl.ONE_MINUS_DST_COLOR,gl.ZERO);gl.colorMask(true,false,false,false);
  gl.bufferData(gl.ARRAY_BUFFER,verts.subarray(0,n),gl.DYNAMIC_DRAW);gl.drawArrays(gl.TRIANGLES,0,n/2);gl.disable(gl.BLEND);
  for(n=0,x=-tiles;x<=tiles;x++)n=disc(n,x*s,0,uR*.085,w,h);gl.uniform4f(flat.uC,0,0,0,1);gl.bufferData(gl.ARRAY_BUFFER,verts.subarray(0,n),gl.DYNAMIC_DRAW);gl.drawArrays(gl.TRIANGLES,0,n/2);
  for(n=0,x=-tiles;x<=tiles;x++)n=disc(n,x*s,0,uR*.06,w,h);gl.colorMask(false,true,false,false);gl.uniform4f(flat.uC,1,1,1,1);gl.bufferData(gl.ARRAY_BUFFER,verts.subarray(0,n),gl.DYNAMIC_DRAW);gl.drawArrays(gl.TRIANGLES,0,n/2);
  gl.colorMask(true,true,true,true);gl.useProgram(down.p);gl.uniform1i(down.uS,0);gl.activeTexture(gl.TEXTURE0);
  [[A,Hf,1],[Hf,B,1],[B,B2,1]].forEach(function(s){into(s[1]);gl.bindTexture(gl.TEXTURE_2D,s[0].tx);gl.uniform2f(down.uO,s[2]/s[0].w,s[2]/s[0].h);quad(FULL);});}
var GL=[].slice.call(document.querySelectorAll('[data-glass]')).reverse(),Q=new Float32Array(8);
function draw(){var w=cv.width,h=cv.height;if(!A)return;if(webT!==t){web();webT=t;}
  into(null);gl.activeTexture(gl.TEXTURE1);gl.bindTexture(gl.TEXTURE_2D,B2.tx);gl.activeTexture(gl.TEXTURE0);gl.bindTexture(gl.TEXTURE_2D,A.tx);
  gl.useProgram(show.p);gl.uniform1i(show.uA,0);gl.uniform3fv(show.uInk,ink);gl.uniform3fv(show.uPaper,paper);gl.uniform3fv(show.uAcc,acc);quad(FULL);
  // each pane of glass, the last in the page first, so an earlier one lies over a later one
  gl.useProgram(glassP.p);gl.uniform1i(glassP.uA,0);gl.uniform1i(glassP.uB,1);gl.uniform2f(glassP.uRes,w,h);gl.uniform1f(glassP.uPx,Math.max(1,d));
  gl.uniform3fv(glassP.uInk,ink);gl.uniform3fv(glassP.uPaper,paper);gl.uniform3fv(glassP.uAcc,acc);gl.enable(gl.BLEND);gl.blendFunc(gl.SRC_ALPHA,gl.ONE_MINUS_SRC_ALPHA);
  var H=cv.clientHeight,m=60*d;GL.forEach(function(el){var r=el.getBoundingClientRect();if(!r.width||r.bottom<-80||r.top>H+80)return;var a=el.getAttribute('data-glass').split(',');
    var cx=(r.left+r.width/2)*d,cy=(H-r.top-r.height/2)*d,hw=r.width/2*d,hh=r.height/2*d;
    gl.uniform4f(glassP.uC,cx,cy,hw,hh);gl.uniform4f(glassP.uP,Math.min(parseFloat(getComputedStyle(el).borderTopLeftRadius)||0,r.height/2,r.width/2)*d,+a[0],+a[1],Math.min(+a[2],r.height/2)*d);
    var x0=(cx-hw-m)/w*2-1,x1=(cx+hw+m)/w*2-1,y0=(cy-hh-m)/h*2-1,y1=(cy+hh+m)/h*2-1;Q[0]=x0;Q[1]=y0;Q[2]=x1;Q[3]=y0;Q[4]=x0;Q[5]=y1;Q[6]=x1;Q[7]=y1;quad(Q);});
  gl.disable(gl.BLEND);}
var due=false;function soon(){if(due)return;due=true;requestAnimationFrame(function(){due=false;draw();});}
window.__redraw=soon;window.__draw=function(){draw();};window.__step=function(){t+=.033;draw();};
if('ResizeObserver' in window){var ro=new ResizeObserver(soon);GL.forEach(function(el){ro.observe(el);});}
// How hard the band works is a ladder: the frame rate first, then the picture's resolution. A frame that was
// drawn and whose next callback came late is a slow one; when most of a batch are slow the band steps down a
// rung, on purpose, so the page stays quick; one bad batch alone, a hitch, changes nothing. After a few seconds on
// time it tries the rung above again; only a try that fails at once makes the next wait longer, and a try that
// holds puts the wait back. Time runs by the clock, so the shapes keep their speed.
var LADDER=[[30,1],[20,1],[15,1],[15,.75],[15,.56],[10,.5]],rung=0,base=16.7,wasDraw=false,late=0,count=0,calm=0,wait=4,batch=0,bad=0,upAt=0,skip=false,born=performance.now(),rate=document.getElementById('rate');
function setRung(k){rung=Math.max(0,Math.min(LADDER.length-1,k));var r=LADDER[rung];if(q!==r[1]){q=r[1];size();}
  rate.textContent=r[0]+' fps'+(r[1]<1?' at '+Math.round(r[1]*100)+'%':'');late=0;count=0;}
function tick(now){if(!on||!seen){raf=0;return;}var dt=now-last;last=now;t+=Math.min(.25,dt/1000)||0;
  // the display's own interval is the shortest gap seen; the first gap after the tab comes back is not counted
  if(!skip&&now-born>700){if(dt>4&&dt<base)base=dt;else base=Math.min(34,base*1.0005);if(!wasDraw){}
    else{count++;if(dt>Math.max(base*1.6+4,30))late++;if(count===1)batch=now;
      if(count>=24||(count>=5&&now-batch>2000)){if(late*2>count){calm=0;late=0;count=0;if(++bad>=2&&rung<LADDER.length-1){bad=0;if(now-upAt<8000)wait=Math.min(32,wait*2);setRung(rung+1);}}
        else{bad=0;if(late<=2)calm+=count/LADDER[rung][0];else calm=0;late=0;count=0;if(now-upAt>20000)wait=4;if(calm>=wait&&rung>0){calm=0;upAt=now;setRung(rung-1);}}}}}
  skip=false;wasDraw=false;if(now-drawn>=1000/LADDER[rung][0]-2){drawn=now;draw();wasDraw=true;}
  raf=requestAnimationFrame(tick);}
document.addEventListener('visibilitychange',function(){skip=true;});
function go(){if(on&&seen&&!raf){skip=true;last=performance.now();raf=requestAnimationFrame(tick);}}
function set(v){on=v;btn.textContent=on?'Pause':'Play';btn.setAttribute('aria-pressed',!on);go();}
if('IntersectionObserver' in window)new IntersectionObserver(function(e){seen=e[0].isIntersecting;go();}).observe(cv);
btn.addEventListener('click',function(){set(!on);});
addEventListener('resize',size);matchMedia('(prefers-color-scheme:dark)').addEventListener('change',size);
size();setRung(0);set(on);if(still)rate.hidden=true;
})();
(function(){
  var still=matchMedia('(prefers-reduced-motion:reduce)').matches,redraw=function(){if(window.__redraw)window.__redraw();};
  // a command's output is there whole; pressing its prompt plays it line by line
  [].slice.call(document.querySelectorAll('.demo')).forEach(function(d){
    var lines=[].slice.call(d.querySelectorAll('.ln[data-s]')),T=[],step=d.hasAttribute('data-fast')?120:340;
    function play(){T.forEach(clearTimeout);T=[];if(still)return;lines.forEach(function(l,i){l.hidden=true;T.push(setTimeout(function(){l.hidden=false;redraw();},step*(i+1)));});redraw();}
    d.querySelector('.ps').addEventListener('click',play);
  });
  // The page is one native scroll, so a wheel, a finger and its momentum all work as anywhere. What it moves is
  // worked out here, each frame: the first stretch of scroll is the pane's rise, one for one, and the next moves the
  // content inside the pane, and the last takes the pane off by the top, over the two links under it. The pane's place, the content's and the glass the canvas draws are set in the same
  // frame, so the rim and the text never part.
  var track=document.getElementById('track'),pane=document.getElementById('pane'),sc=document.getElementById('scroll'),body=document.getElementById('content'),
      spacer=document.getElementById('spacer'),end=document.getElementById('end'),T=1,over=0,PB=0,g=-1,inner=-1,e=-1,due=0;
  function paint(){due=0;var y=track.scrollTop,v=Math.max(0,Math.min(1,y/T)),n=Math.max(0,Math.min(over,Math.round(y-T))),x=Math.max(0,Math.round(y-T-over));
    if(x!==e){e=x;pane.style.setProperty('--e',x);end.style.visibility=x?'visible':'hidden';end.style.transform='translateY('+Math.max(0,PB-x+60-innerHeight*.8)+'px)';g=-1;}
    if(n!==inner){inner=n;body.style.transform='translateY('+(-n)+'px)';}
    if(v!==g){g=v;pane.style.setProperty('--g',v.toFixed(4));if(window.__draw)window.__draw();}}
  function ask(){if(!due)due=requestAnimationFrame(paint);}
  // how far the pane rises, and how much of the content lies below the risen pane
  function measure(){T=Math.round(innerWidth<=820?innerHeight*.52+52:innerHeight*.56-66);
    pane.style.setProperty('--g','1');pane.style.setProperty('--e',0);var cs=getComputedStyle(sc);over=Math.max(0,Math.ceil(body.offsetHeight+parseFloat(cs.paddingTop)+parseFloat(cs.paddingBottom)-sc.clientHeight));
    PB=pane.getBoundingClientRect().bottom;spacer.style.height=(T+over+Math.ceil(PB)+40)+'px';g=-1;inner=-1;e=-1;paint();}
  track.addEventListener('scroll',ask,{passive:true});addEventListener('resize',measure);
  if(document.fonts&&document.fonts.ready)document.fonts.ready.then(measure);measure();
})();
