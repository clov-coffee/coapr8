const fs = require('fs')

const readdir = d => fs.readdirSync(d)
                       .reduce( (a, de) => !fs.statSync(d + '/' + de).isDirectory()
                                         ? [...a, d + '/' + de]
                                         : [...a, ...readdir(d + '/' + de)]
                              , []
                              );

readdir('.').filter(n => !n.includes('target') && n.includes('examples')).forEach(p => {
  console.log(p);
  let c = fs.readFileSync(p, 'utf8');
  c = c.replace(/kwap/g, 'toad')
  fs.writeFileSync(p, c, 'utf8')
});
