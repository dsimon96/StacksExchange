import React from 'react';
import logo from './logo.svg';
import NavigationBar from './components/Navigation/NavigationBar';

class App extends React.Component {

  componentDidMount() {
    const script = document.createElement("script");
    script.async = true;
    script.defer = true;
    script.src = "https://accounts.google.com/gsi/client";
    document.body.appendChild(script)
  }

  render() {
    return (
      <div className="App">
        <header>
          <div className="g_id_signout">{/* Signout is implemented here. */}</div>
        </header>
        <div id="g_id_onload"
          data-client_id="962633347992-tbgvt8rcmnhdp5tlfm2hs1av8bkfc03n.apps.googleusercontent.com"
          data-login_uri="http://localhost:8080/oauth">
        </div>

        <NavigationBar />
      </div>
    );
  }
}

export default App;
